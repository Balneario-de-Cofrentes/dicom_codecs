use rustler::{Binary, Env, NifMap, NifResult, OwnedBinary};
use std::io::Write;

mod jpeg;
mod jpeg2000;
mod jpegls;

/// Image metadata passed from Elixir as a map.
#[derive(Debug, NifMap)]
pub struct Metadata {
    pub rows: u32,
    pub columns: u32,
    pub bits_allocated: u32,
    pub bits_stored: u32,
    pub samples_per_pixel: u32,
    pub photometric_interpretation: String,
    pub pixel_representation: u32,
}

// -- JPEG --

#[rustler::nif]
fn jpeg_decode<'a>(env: Env<'a>, data: Binary<'a>, meta: Metadata) -> NifResult<Binary<'a>> {
    let raw = jpeg::decode(data.as_slice(), &meta)
        .map_err(|e| rustler::Error::Term(Box::new(format!("jpeg_decode: {e}"))))?;

    let mut out = OwnedBinary::new(raw.len()).ok_or(rustler::Error::Atom("alloc_failed"))?;
    out.as_mut_slice().write_all(&raw).map_err(|_| rustler::Error::Atom("write_failed"))?;
    Ok(out.release(env))
}

#[rustler::nif]
fn jpeg_encode<'a>(env: Env<'a>, data: Binary<'a>, meta: Metadata) -> NifResult<Binary<'a>> {
    let compressed = jpeg::encode(data.as_slice(), &meta)
        .map_err(|e| rustler::Error::Term(Box::new(format!("jpeg_encode: {e}"))))?;

    let mut out =
        OwnedBinary::new(compressed.len()).ok_or(rustler::Error::Atom("alloc_failed"))?;
    out.as_mut_slice()
        .write_all(&compressed)
        .map_err(|_| rustler::Error::Atom("write_failed"))?;
    Ok(out.release(env))
}

// -- JPEG 2000 --

#[rustler::nif]
fn jpeg2000_decode<'a>(env: Env<'a>, data: Binary<'a>, meta: Metadata) -> NifResult<Binary<'a>> {
    let raw = jpeg2000::decode(data.as_slice(), &meta)
        .map_err(|e| rustler::Error::Term(Box::new(format!("jpeg2000_decode: {e}"))))?;

    let mut out = OwnedBinary::new(raw.len()).ok_or(rustler::Error::Atom("alloc_failed"))?;
    out.as_mut_slice().write_all(&raw).map_err(|_| rustler::Error::Atom("write_failed"))?;
    Ok(out.release(env))
}

#[rustler::nif]
fn jpeg2000_encode<'a>(env: Env<'a>, data: Binary<'a>, meta: Metadata) -> NifResult<Binary<'a>> {
    let compressed = jpeg2000::encode(data.as_slice(), &meta)
        .map_err(|e| rustler::Error::Term(Box::new(format!("jpeg2000_encode: {e}"))))?;

    let mut out =
        OwnedBinary::new(compressed.len()).ok_or(rustler::Error::Atom("alloc_failed"))?;
    out.as_mut_slice()
        .write_all(&compressed)
        .map_err(|_| rustler::Error::Atom("write_failed"))?;
    Ok(out.release(env))
}

// -- JPEG-LS --

#[rustler::nif]
fn jpegls_decode<'a>(env: Env<'a>, data: Binary<'a>, meta: Metadata) -> NifResult<Binary<'a>> {
    let raw = jpegls::decode(data.as_slice(), &meta)
        .map_err(|e| rustler::Error::Term(Box::new(format!("jpegls_decode: {e}"))))?;

    let mut out = OwnedBinary::new(raw.len()).ok_or(rustler::Error::Atom("alloc_failed"))?;
    out.as_mut_slice().write_all(&raw).map_err(|_| rustler::Error::Atom("write_failed"))?;
    Ok(out.release(env))
}

#[rustler::nif]
fn jpegls_encode<'a>(env: Env<'a>, data: Binary<'a>, meta: Metadata) -> NifResult<Binary<'a>> {
    let compressed = jpegls::encode(data.as_slice(), &meta)
        .map_err(|e| rustler::Error::Term(Box::new(format!("jpegls_encode: {e}"))))?;

    let mut out =
        OwnedBinary::new(compressed.len()).ok_or(rustler::Error::Atom("alloc_failed"))?;
    out.as_mut_slice()
        .write_all(&compressed)
        .map_err(|_| rustler::Error::Atom("write_failed"))?;
    Ok(out.release(env))
}

rustler::init!("Elixir.DicomCodecs.Native");
