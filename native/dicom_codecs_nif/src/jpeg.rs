use crate::error::CodecError;
use crate::frame::{Image, Metadata};
use jpeg_decoder::PixelFormat;
use jpeg_encoder::ColorType;

/// Quality of the lossy baseline encoder (1–100). From 90 up `jpeg-encoder`
/// keeps full chroma resolution (4:4:4).
const ENCODE_QUALITY: u8 = 95;

/// Decodes a JPEG frame with `jpeg-decoder`.
///
/// Handles 8-bit baseline and extended (lossy) streams, and lossless streams
/// (process 14, any predictor) of 2 to 16 bits. Three-component lossy streams
/// are converted to RGB; lossless samples are returned as stored. 12-bit lossy
/// streams (extended process 4) are rejected by the decoder.
pub fn decode(data: &[u8], meta: &Metadata) -> Result<Image, CodecError> {
    let mut decoder = jpeg_decoder::Decoder::new(data);
    // Refuse to allocate more samples than the frame the metadata describes.
    decoder.set_max_decoding_buffer_size(
        meta.rows as usize * meta.columns as usize * meta.samples_per_pixel as usize,
    );

    let pixels = decoder
        .decode()
        .map_err(|e| CodecError::DecodeFailed(format!("JPEG: {e}")))?;
    let info = decoder
        .info()
        .ok_or_else(|| CodecError::Internal("JPEG: no image info after decode".into()))?;

    let components = match info.pixel_format {
        PixelFormat::L8 | PixelFormat::L16 => 1,
        PixelFormat::RGB24 => 3,
        PixelFormat::CMYK32 => return Err(CodecError::Unsupported("JPEG: CMYK streams".into())),
    };
    // `pixel_format` says RGB24 even for 16-bit lossless colour, so derive the
    // sample width from the buffer instead.
    let samples = info.width as usize * info.height as usize * components as usize;
    let bytes_per_sample = pixels.len().checked_div(samples).unwrap_or(0);

    Ok(Image {
        width: info.width.into(),
        height: info.height.into(),
        components,
        bytes_per_sample,
        data: pixels,
    })
}

/// Encodes an 8-bit greyscale or RGB frame as lossy baseline JPEG (process 1).
pub fn encode(raw: &[u8], meta: &Metadata) -> Result<Vec<u8>, CodecError> {
    if meta.bits_allocated != 8 {
        return Err(CodecError::Unsupported(format!(
            "JPEG baseline encoding needs 8-bit samples, got bits_allocated {}",
            meta.bits_allocated
        )));
    }
    let color_type = match meta.samples_per_pixel {
        1 => ColorType::Luma,
        _ => ColorType::Rgb,
    };

    let mut output = Vec::new();
    jpeg_encoder::Encoder::new(&mut output, ENCODE_QUALITY)
        // validate() caps rows and columns at u16::MAX.
        .encode(raw, meta.columns as u16, meta.rows as u16, color_type)
        .map_err(|e| CodecError::EncodeFailed(format!("JPEG: {e}")))?;
    Ok(output)
}
