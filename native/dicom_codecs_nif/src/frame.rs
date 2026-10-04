use crate::error::CodecError;
use rustler::{NifMap, Term};

/// Image parameters of one frame, decoded from the Elixir metadata map.
#[derive(Debug, NifMap)]
pub struct Metadata {
    pub rows: u32,
    pub columns: u32,
    pub bits_allocated: u32,
    pub bits_stored: u32,
    pub samples_per_pixel: u32,
    pub photometric_interpretation: String,
    pub pixel_representation: u32,
    pub planar_configuration: u32,
}

/// A decoded frame: samples interleaved by pixel, little-endian, unpadded rows.
#[derive(Debug)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub components: u32,
    pub bytes_per_sample: usize,
    pub data: Vec<u8>,
}

impl Metadata {
    pub fn from_term(term: Term<'_>) -> Result<Self, CodecError> {
        let meta: Metadata = term.decode().map_err(|_| {
            CodecError::InvalidMetadata(
                "expected a map with non-negative integers rows, columns, bits_allocated, \
                 bits_stored, samples_per_pixel, pixel_representation, planar_configuration \
                 and a string photometric_interpretation"
                    .into(),
            )
        })?;
        meta.validate()?;
        Ok(meta)
    }

    fn validate(&self) -> Result<(), CodecError> {
        let invalid = |m: String| Err(CodecError::InvalidMetadata(m));

        // Rows and Columns are US (16-bit) attributes in DICOM.
        if !(1..=u16::MAX as u32).contains(&self.rows)
            || !(1..=u16::MAX as u32).contains(&self.columns)
        {
            return invalid(format!(
                "rows and columns must be between 1 and 65535, got {}x{}",
                self.rows, self.columns
            ));
        }
        if !matches!(self.samples_per_pixel, 1 | 3) {
            return invalid(format!(
                "samples_per_pixel must be 1 or 3, got {}",
                self.samples_per_pixel
            ));
        }
        if !matches!(self.bits_allocated, 8 | 16) {
            return invalid(format!(
                "bits_allocated must be 8 or 16, got {}",
                self.bits_allocated
            ));
        }
        if self.bits_stored == 0 || self.bits_stored > self.bits_allocated {
            return invalid(format!(
                "bits_stored must be between 1 and bits_allocated ({}), got {}",
                self.bits_allocated, self.bits_stored
            ));
        }
        if self.pixel_representation > 1 {
            return invalid(format!(
                "pixel_representation must be 0 or 1, got {}",
                self.pixel_representation
            ));
        }
        if self.planar_configuration > 1 {
            return invalid(format!(
                "planar_configuration must be 0 or 1, got {}",
                self.planar_configuration
            ));
        }
        if self.planar_configuration == 1 && self.samples_per_pixel > 1 {
            return Err(CodecError::Unsupported(
                "planar_configuration 1 (planes stored separately) is not supported".into(),
            ));
        }
        Ok(())
    }

    pub fn bytes_per_sample(&self) -> usize {
        self.bits_allocated as usize / 8
    }

    pub fn is_signed(&self) -> bool {
        self.pixel_representation == 1
    }

    /// Size in bytes of one uncompressed frame.
    pub fn frame_len(&self) -> usize {
        // validate() caps this at 65535² × 3 × 2 bytes; lib.rs requires a 64-bit usize.
        self.rows as usize
            * self.columns as usize
            * self.samples_per_pixel as usize
            * self.bytes_per_sample()
    }

    pub fn check_raw_len(&self, len: usize) -> Result<(), CodecError> {
        let expected = self.frame_len();
        if len == expected {
            Ok(())
        } else {
            Err(CodecError::MetadataMismatch(format!(
                "pixel data is {len} bytes, metadata describes {expected} \
                 ({}x{}, {} samples of {} bytes)",
                self.columns,
                self.rows,
                self.samples_per_pixel,
                self.bytes_per_sample()
            )))
        }
    }

    /// Compares a stream's declared geometry with the metadata. Codecs call it
    /// before allocating the output, since a corrupt header can declare any size.
    pub fn check_header(
        &self,
        width: u32,
        height: u32,
        components: u32,
        bytes_per_sample: usize,
    ) -> Result<(), CodecError> {
        let expected = (
            self.columns,
            self.rows,
            self.samples_per_pixel,
            self.bytes_per_sample(),
        );
        if (width, height, components, bytes_per_sample) == expected {
            return Ok(());
        }
        let (columns, rows, samples, sample_bytes) = expected;
        Err(CodecError::MetadataMismatch(format!(
            "frame is {width}x{height} with {components} samples of {bytes_per_sample} bytes, \
             metadata describes {columns}x{rows} with {samples} samples of {sample_bytes} bytes"
        )))
    }

    pub fn check_decoded(&self, image: &Image) -> Result<(), CodecError> {
        self.check_header(
            image.width,
            image.height,
            image.components,
            image.bytes_per_sample,
        )?;
        // Codecs build `data` from the same dimensions, so this only guards against
        // a codec bug handing back a short or long buffer.
        if image.data.len() != self.frame_len() {
            return Err(CodecError::Internal(format!(
                "decoder returned {} bytes for a {}-byte frame",
                image.data.len(),
                self.frame_len()
            )));
        }
        Ok(())
    }

    /// Stored sample values of a raw frame: bits above `bits_stored` are masked
    /// off and signed values are sign-extended, as PS3.5 §8.1.1 prescribes.
    pub fn samples(&self, raw: &[u8]) -> Vec<i32> {
        // validate() bounds bits_stored to 1..=16, so none of these shifts overflow.
        let bits = self.bits_stored;
        let mask = (1u32 << bits) - 1;
        let sign_bit = 1u32 << (bits - 1);
        let stored = |raw: u32| {
            let value = raw & mask;
            if self.is_signed() && value & sign_bit != 0 {
                value as i32 - (1i32 << bits)
            } else {
                value as i32
            }
        };

        match self.bytes_per_sample() {
            1 => raw.iter().map(|&b| stored(b as u32)).collect(),
            _ => raw
                .chunks_exact(2)
                .map(|c| stored(u16::from_le_bytes([c[0], c[1]]) as u32))
                .collect(),
        }
    }
}

/// Writes signed or unsigned sample values as little-endian bytes of the given width.
pub fn samples_to_bytes(samples: impl Iterator<Item = i32>, bytes_per_sample: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.size_hint().0 * bytes_per_sample);
    match bytes_per_sample {
        1 => out.extend(samples.map(|v| v as u8)),
        _ => samples.for_each(|v| out.extend_from_slice(&(v as u16).to_le_bytes())),
    }
    out
}
