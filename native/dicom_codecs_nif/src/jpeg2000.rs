use crate::Metadata;

/// Decode a JPEG 2000 frame to raw pixel data.
///
/// Uses OpenJPEG via the `openjp2` crate. Handles both lossless (J2K)
/// and lossy JPEG 2000 streams. The decoder auto-detects the format
/// from the codestream header (J2K or JP2 container).
pub fn decode(data: &[u8], _meta: &Metadata) -> Result<Vec<u8>, String> {
    let image = openjp2::decode_to_image(data)
        .map_err(|e| format!("JPEG 2000 decode failed: {e}"))?;

    // Convert openjp2 image components to interleaved raw bytes
    let num_comps = image.num_components();
    if num_comps == 0 {
        return Err("JPEG 2000: no components in decoded image".into());
    }

    let comp0 = &image.components()[0];
    let width = comp0.width() as usize;
    let height = comp0.height() as usize;
    let prec = comp0.precision() as usize;
    let bytes_per_sample = if prec <= 8 { 1 } else { 2 };
    let pixel_count = width * height;

    let mut output = Vec::with_capacity(pixel_count * num_comps * bytes_per_sample);

    if bytes_per_sample == 1 {
        // 8-bit: interleave components
        for i in 0..pixel_count {
            for c in 0..num_comps {
                let val = image.components()[c].data()[i];
                output.push(val as u8);
            }
        }
    } else {
        // 16-bit: interleave as little-endian u16
        for i in 0..pixel_count {
            for c in 0..num_comps {
                let val = image.components()[c].data()[i] as u16;
                output.extend_from_slice(&val.to_le_bytes());
            }
        }
    }

    Ok(output)
}

/// Encode raw pixels to JPEG 2000 lossless.
pub fn encode(raw_pixels: &[u8], meta: &Metadata) -> Result<Vec<u8>, String> {
    let width = meta.columns;
    let height = meta.rows;
    let num_comps = meta.samples_per_pixel;
    let prec = meta.bits_stored;
    let _bytes_per_sample = if prec <= 8 { 1usize } else { 2 };

    // Build component parameters
    let mut params = openjp2::CompressionParameters::default();
    params.tcp_numlayers = 1;
    params.cp_disto_alloc = 1;
    params.tcp_rates[0] = 0.0; // lossless

    let image = openjp2::encode_to_j2k(
        raw_pixels,
        width,
        height,
        num_comps,
        prec,
        meta.pixel_representation != 0, // signed
        &params,
    )
    .map_err(|e| format!("JPEG 2000 encode failed: {e}"))?;

    Ok(image)
}
