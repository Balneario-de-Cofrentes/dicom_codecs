use crate::Metadata;

/// Decode a JPEG-LS frame to raw pixel data.
///
/// Uses CharLS for both lossless and near-lossless JPEG-LS decoding.
/// The NEAR parameter (lossy tolerance) is embedded in the bitstream.
pub fn decode(data: &[u8], _meta: &Metadata) -> Result<Vec<u8>, String> {
    let decoded = charls::decode(data).map_err(|e| format!("JPEG-LS decode failed: {e}"))?;
    Ok(decoded.data)
}

/// Encode raw pixels to JPEG-LS lossless.
pub fn encode(raw_pixels: &[u8], meta: &Metadata) -> Result<Vec<u8>, String> {
    let width = meta.columns;
    let height = meta.rows;
    let num_comps = meta.samples_per_pixel;
    let bits_per_sample = meta.bits_stored;

    let params = charls::EncodeParams {
        width,
        height,
        components: num_comps,
        bits_per_sample,
        near_lossless: 0, // lossless
        interleave_mode: if num_comps > 1 {
            charls::InterleaveMode::Sample
        } else {
            charls::InterleaveMode::None
        },
    };

    let compressed =
        charls::encode(raw_pixels, &params).map_err(|e| format!("JPEG-LS encode failed: {e}"))?;

    Ok(compressed)
}
