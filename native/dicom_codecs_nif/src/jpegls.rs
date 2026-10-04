//! JPEG-LS through CharLS (`charls-sys`).

use crate::error::CodecError;
use crate::frame::{self, Image, Metadata};
use charls_sys::*;
use std::borrow::Cow;
use std::ffi::{c_void, CStr};

// Exported by CharLS 2.4 but missing from the charls-sys bindings.
extern "C" {
    fn charls_jpegls_decoder_get_interleave_mode(
        decoder: *const charls_jpegls_decoder,
        interleave_mode: *mut charls_interleave_mode,
    ) -> charls_jpegls_errc;
}

/// Decodes a lossless or near-lossless JPEG-LS frame. Colour frames are
/// returned interleaved by pixel whatever the interleave mode of the stream.
pub fn decode(data: &[u8], meta: &Metadata) -> Result<Image, CodecError> {
    let failed = |code| CodecError::DecodeFailed(format!("JPEG-LS: {}", message(code)));
    let decoder = DecoderHandle::new().map_err(failed)?;
    let data = end_with_marker(data);
    let (info, interleave) = read_header(&decoder, &data).map_err(failed)?;

    let components = info.component_count as u32;
    let bytes_per_sample = if info.bits_per_sample <= 8 { 1 } else { 2 };
    // Check before allocating: a corrupt header can declare any size.
    // This also covers GHSA-mqrg-gfc8-73ff (width overflow in CharLS < 2.4.4):
    // the advisory's workaround is to check dimensions between header and decode.
    meta.check_header(info.width, info.height, components, bytes_per_sample)?;

    let mut pixels = vec![0u8; meta.frame_len()];
    unsafe {
        check(charls_jpegls_decoder_decode_to_buffer(
            decoder.0,
            pixels.as_mut_ptr() as *mut c_void,
            pixels.len(),
            0,
        ))
        .map_err(failed)?;
    }
    if interleave == charls_interleave_mode_none && components > 1 {
        pixels = planar_to_interleaved(&pixels, components as usize, bytes_per_sample);
    }

    Ok(Image {
        width: info.width,
        height: info.height,
        components,
        bytes_per_sample,
        data: pixels,
    })
}

/// Points the decoder at `data` and reads the frame info and interleave mode.
/// `data` must outlive the decoder's use of it.
fn read_header(
    decoder: &DecoderHandle,
    data: &[u8],
) -> Result<(charls_frame_info, charls_interleave_mode), charls_jpegls_errc> {
    let mut info = charls_frame_info {
        width: 0,
        height: 0,
        bits_per_sample: 0,
        component_count: 0,
    };
    let mut interleave = charls_interleave_mode_none;
    unsafe {
        check(charls_jpegls_decoder_set_source_buffer(
            decoder.0,
            data.as_ptr() as *const c_void,
            data.len(),
        ))?;
        check(charls_jpegls_decoder_read_header(decoder.0))?;
        check(charls_jpegls_decoder_get_frame_info(decoder.0, &mut info))?;
        check(charls_jpegls_decoder_get_interleave_mode(
            decoder.0,
            &mut interleave,
        ))?;
    }
    Ok((info, interleave))
}

/// Appends an EOI marker to a stream that does not end with a marker.
///
/// CharLS 2.4.2 (and 2.4.4) only reports running out of data at the end of
/// the buffer when its bit count is exactly zero; once it is negative the
/// unary-code reader spins reading zeros until an `int32_t` wraps, about 2.5 s
/// per truncated or corrupt frame. At a marker the same check uses `<= 0`, and
/// the reader never reads past a marker, so ending the buffer with one turns
/// that spin into an immediate error.
fn end_with_marker(data: &[u8]) -> Cow<'_, [u8]> {
    match data {
        [.., 0xFF, code] if code & 0x80 != 0 => data.into(),
        _ => [data, &[0xFF, 0xD9]].concat().into(),
    }
}

/// Encodes a frame as lossless JPEG-LS, sample-interleaved for colour frames.
///
/// JPEG-LS has no notion of signed samples: signed frames are encoded as their
/// two's-complement bit patterns, as DICOM implementations conventionally do.
pub fn encode(raw: &[u8], meta: &Metadata) -> Result<Vec<u8>, CodecError> {
    if meta.bits_stored < 2 {
        return Err(CodecError::Unsupported(format!(
            "JPEG-LS needs at least 2 bits per sample, got bits_stored {}",
            meta.bits_stored
        )));
    }
    // CharLS rejects samples wider than bits_per_sample, so drop the bits above
    // bits_stored that DICOM allows to carry other data.
    let input = mask_to_bits_stored(raw, meta);
    let info = charls_frame_info {
        width: meta.columns,
        height: meta.rows,
        bits_per_sample: meta.bits_stored as i32,
        component_count: meta.samples_per_pixel as i32,
    };

    // CharLS's size estimate covers compressible data only; noise can expand
    // past it, so retry once with the worst case the standard allows.
    match encode_into(&input, &info, None) {
        Err(code) if code == charls_jpegls_errc_destination_buffer_too_small => {
            encode_into(&input, &info, Some(worst_case_size(meta)))
        }
        result => result,
    }
    .map_err(|code| CodecError::EncodeFailed(format!("JPEG-LS: {}", message(code))))
}

/// Encodes into a buffer of `capacity` bytes, or CharLS's estimate if `None`.
fn encode_into(
    input: &[u8],
    info: &charls_frame_info,
    capacity: Option<usize>,
) -> Result<Vec<u8>, charls_jpegls_errc> {
    let encoder = EncoderHandle::new()?;
    let interleave = match info.component_count {
        1 => charls_interleave_mode_none,
        _ => charls_interleave_mode_sample,
    };

    let mut size = 0usize;
    unsafe {
        check(charls_jpegls_encoder_set_frame_info(encoder.0, info))?;
        check(charls_jpegls_encoder_set_interleave_mode(
            encoder.0, interleave,
        ))?;
        check(charls_jpegls_encoder_set_near_lossless(encoder.0, 0))?;
        check(charls_jpegls_encoder_get_estimated_destination_size(
            encoder.0, &mut size,
        ))?;
    }
    let mut output = vec![0u8; capacity.unwrap_or(size)];
    unsafe {
        check(charls_jpegls_encoder_set_destination_buffer(
            encoder.0,
            output.as_mut_ptr() as *mut c_void,
            output.len(),
        ))?;
        check(charls_jpegls_encoder_encode_from_buffer(
            encoder.0,
            input.as_ptr() as *const c_void,
            input.len(),
            0,
        ))?;
        check(charls_jpegls_encoder_get_bytes_written(
            encoder.0, &mut size,
        ))?;
    }
    output.truncate(size);
    Ok(output)
}

/// Upper bound of a lossless JPEG-LS stream: every sample coded with the
/// longest code, LIMIT = 2·(bits + max(8, bits)) bits (ISO/IEC 14495-1 A.2.1),
/// with a stuffed bit after every byte (7 payload bits per byte), plus markers.
fn worst_case_size(meta: &Metadata) -> usize {
    let bits = meta.bits_stored as usize;
    let limit = 2 * (bits + bits.max(8));
    let samples = meta.frame_len() / meta.bytes_per_sample();
    (samples * limit).div_ceil(7) + 4096
}

fn mask_to_bits_stored<'a>(raw: &'a [u8], meta: &Metadata) -> Cow<'a, [u8]> {
    if meta.bits_stored == meta.bits_allocated {
        return raw.into();
    }
    let mask = (1i32 << meta.bits_stored) - 1;
    let samples = meta.samples(raw).into_iter().map(|v| v & mask);
    frame::samples_to_bytes(samples, meta.bytes_per_sample()).into()
}

/// Reorders `RRR…GGG…BBB…` into `RGBRGB…`.
fn planar_to_interleaved(planar: &[u8], components: usize, bytes_per_sample: usize) -> Vec<u8> {
    let plane_len = planar.len() / components;
    let mut out = Vec::with_capacity(planar.len());
    for offset in (0..plane_len).step_by(bytes_per_sample) {
        for c in 0..components {
            let start = c * plane_len + offset;
            out.extend_from_slice(&planar[start..start + bytes_per_sample]);
        }
    }
    out
}

fn check(code: charls_jpegls_errc) -> Result<(), charls_jpegls_errc> {
    match code {
        0 => Ok(()),
        error => Err(error),
    }
}

fn message(code: charls_jpegls_errc) -> String {
    unsafe { CStr::from_ptr(charls_get_error_message(code)) }
        .to_string_lossy()
        .into_owned()
}

struct DecoderHandle(*mut charls_jpegls_decoder);

impl DecoderHandle {
    fn new() -> Result<Self, charls_jpegls_errc> {
        let ptr = unsafe { charls_jpegls_decoder_create() };
        if ptr.is_null() {
            Err(charls_jpegls_errc_not_enough_memory)
        } else {
            Ok(Self(ptr))
        }
    }
}

impl Drop for DecoderHandle {
    fn drop(&mut self) {
        unsafe { charls_jpegls_decoder_destroy(self.0) }
    }
}

struct EncoderHandle(*mut charls_jpegls_encoder);

impl EncoderHandle {
    fn new() -> Result<Self, charls_jpegls_errc> {
        let ptr = unsafe { charls_jpegls_encoder_create() };
        if ptr.is_null() {
            Err(charls_jpegls_errc_not_enough_memory)
        } else {
            Ok(Self(ptr))
        }
    }
}

impl Drop for EncoderHandle {
    fn drop(&mut self) {
        unsafe { charls_jpegls_encoder_destroy(self.0) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata(samples_per_pixel: u32, bits: u32) -> Metadata {
        Metadata {
            rows: 3,
            columns: 5,
            bits_allocated: if bits > 8 { 16 } else { 8 },
            bits_stored: bits,
            samples_per_pixel,
            photometric_interpretation: "RGB".into(),
            pixel_representation: 0,
            planar_configuration: 0,
        }
    }

    /// Encodes with an explicit interleave mode, which the NIF API never picks
    /// for colour frames other than `sample`.
    fn encode_with(raw: &[u8], meta: &Metadata, mode: charls_interleave_mode) -> Vec<u8> {
        let encoder = EncoderHandle::new().unwrap();
        let info = charls_frame_info {
            width: meta.columns,
            height: meta.rows,
            bits_per_sample: meta.bits_stored as i32,
            component_count: meta.samples_per_pixel as i32,
        };
        let mut size = 0;
        unsafe {
            check(charls_jpegls_encoder_set_frame_info(encoder.0, &info)).unwrap();
            check(charls_jpegls_encoder_set_interleave_mode(encoder.0, mode)).unwrap();
            check(charls_jpegls_encoder_get_estimated_destination_size(
                encoder.0, &mut size,
            ))
            .unwrap();
            let mut out = vec![0u8; size];
            check(charls_jpegls_encoder_set_destination_buffer(
                encoder.0,
                out.as_mut_ptr() as *mut c_void,
                out.len(),
            ))
            .unwrap();
            check(charls_jpegls_encoder_encode_from_buffer(
                encoder.0,
                raw.as_ptr() as *const c_void,
                raw.len(),
                0,
            ))
            .unwrap();
            check(charls_jpegls_encoder_get_bytes_written(
                encoder.0, &mut size,
            ))
            .unwrap();
            out.truncate(size);
            out
        }
    }

    #[test]
    fn decodes_every_interleave_mode_to_pixel_interleaved_samples() {
        for bits in [8, 16] {
            let meta = metadata(3, bits);
            let interleaved: Vec<u8> = (0..meta.frame_len()).map(|i| (i * 7 % 251) as u8).collect();
            let samples = meta.frame_len() / meta.bytes_per_sample();
            let bps = meta.bytes_per_sample();
            // CharLS expects planes (RRR…GGG…BBB…) for interleave mode none.
            let planar: Vec<u8> = (0..3)
                .flat_map(|c| (0..samples / 3).map(move |p| (p * 3 + c) * bps))
                .flat_map(|at| interleaved[at..at + bps].to_vec())
                .collect();

            for (mode, input) in [
                (charls_interleave_mode_none, &planar),
                (charls_interleave_mode_line, &interleaved),
                (charls_interleave_mode_sample, &interleaved),
            ] {
                let stream = encode_with(input, &meta, mode);
                let image = decode(&stream, &meta).unwrap();
                assert_eq!(
                    image.data, interleaved,
                    "{bits}-bit, interleave mode {mode}"
                );
            }
        }
    }

    #[test]
    fn terminates_streams_that_do_not_end_with_a_marker() {
        assert_eq!(&*end_with_marker(&[1, 0xFF, 0xD9]), &[1, 0xFF, 0xD9]);
        assert_eq!(
            &*end_with_marker(&[1, 0xFF, 0xD9, 0]),
            &[1, 0xFF, 0xD9, 0, 0xFF, 0xD9]
        );
        assert_eq!(&*end_with_marker(&[1, 0xFF]), &[1, 0xFF, 0xFF, 0xD9]);
        assert_eq!(&*end_with_marker(&[]), &[0xFF, 0xD9]);
    }
}
