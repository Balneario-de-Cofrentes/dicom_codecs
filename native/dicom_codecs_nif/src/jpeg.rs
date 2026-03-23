use crate::Metadata;
use std::io::Cursor;

/// Decode a JPEG frame to raw pixel data.
///
/// Handles 8-bit and 12/16-bit lossless JPEG. The decoder auto-detects
/// the JPEG process from the SOF marker in the bitstream.
pub fn decode(data: &[u8], _meta: &Metadata) -> Result<Vec<u8>, String> {
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(data));
    let pixels = decoder.decode().map_err(|e| format!("JPEG decode failed: {e}"))?;

    let info = decoder.info().ok_or("JPEG: no image info after decode")?;

    // For 16-bit JPEG (lossless), pixels are already in native byte order
    // For 8-bit, pixels are raw bytes
    // Return as-is — the caller (Dicom.PixelData) handles photometric conversion
    let _ = info; // metadata available if needed for future photometric conversion
    Ok(pixels)
}

/// Encode raw pixels to JPEG baseline.
///
/// Uses 8-bit baseline JPEG with quality 95. For DICOM, lossy JPEG
/// encoding is typically used for display/web purposes, not archival.
pub fn encode(raw_pixels: &[u8], meta: &Metadata) -> Result<Vec<u8>, String> {
    let width = meta.columns as u16;
    let height = meta.rows as u16;

    let color_type = match meta.samples_per_pixel {
        3 => jpeg_encoder::ColorType::Rgb,
        1 => jpeg_encoder::ColorType::Luma,
        n => return Err(format!("JPEG encode: unsupported samples_per_pixel={n}")),
    };

    let encoder = jpeg_encoder::Encoder::new_file(
        &mut Vec::new(), // temporary, we'll use encode_to_vec
        95,
    )
    .map_err(|e| format!("JPEG encoder init: {e}"))?;

    // Use a buffer approach
    let mut output = Vec::new();
    let enc = jpeg_encoder::Encoder::new(&mut output, 95)
        .map_err(|e| format!("JPEG encoder init: {e}"))?;

    enc.encode(raw_pixels, width, height, color_type)
        .map_err(|e| format!("JPEG encode failed: {e}"))?;

    Ok(output)
}
