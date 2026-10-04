//! Rustler NIFs for DICOM pixel data codecs.
//!
//! Every NIF decodes or encodes a whole frame, which takes far longer than the
//! 1 ms budget of a normal scheduler, so all of them run on dirty CPU schedulers.
//! They return `{:ok, binary}` or `{:error, {reason, message}}`; a panic inside a
//! codec becomes `{:error, {:internal_error, message}}`.

use rustler::{Binary, Env, Term};

mod error;
mod frame;
mod jpeg;
mod jpeg2000;
mod jpegls;

use error::CodecError;
use frame::{Image, Metadata};

// DICOM native pixel data is little-endian and codecs exchange samples in host
// order; frame sizes (up to 65535² × 3 × 2 bytes) need a 64-bit usize.
#[cfg(any(target_endian = "big", not(target_pointer_width = "64")))]
compile_error!("dicom_codecs_nif supports 64-bit little-endian targets only");

type Decoder = fn(&[u8], &Metadata) -> Result<Image, CodecError>;
type Encoder = fn(&[u8], &Metadata) -> Result<Vec<u8>, CodecError>;

#[rustler::nif(schedule = "DirtyCpu")]
fn jpeg_decode<'a>(env: Env<'a>, data: Binary<'a>, meta: Term<'a>) -> Term<'a> {
    decode_frame(env, &data, meta, jpeg::decode)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn jpeg_encode<'a>(env: Env<'a>, data: Binary<'a>, meta: Term<'a>) -> Term<'a> {
    encode_frame(env, &data, meta, jpeg::encode)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn jpeg2000_decode<'a>(env: Env<'a>, data: Binary<'a>, meta: Term<'a>) -> Term<'a> {
    decode_frame(env, &data, meta, jpeg2000::decode)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn jpeg2000_encode<'a>(env: Env<'a>, data: Binary<'a>, meta: Term<'a>) -> Term<'a> {
    encode_frame(env, &data, meta, jpeg2000::encode)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn jpegls_decode<'a>(env: Env<'a>, data: Binary<'a>, meta: Term<'a>) -> Term<'a> {
    decode_frame(env, &data, meta, jpegls::decode)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn jpegls_encode<'a>(env: Env<'a>, data: Binary<'a>, meta: Term<'a>) -> Term<'a> {
    encode_frame(env, &data, meta, jpegls::encode)
}

fn decode_frame<'a>(env: Env<'a>, data: &[u8], meta: Term<'a>, decoder: Decoder) -> Term<'a> {
    let result = Metadata::from_term(meta).and_then(|meta| {
        let image = error::catch_panic(|| decoder(data, &meta))?;
        meta.check_decoded(&image)?;
        Ok(image.data)
    });
    reply(env, result)
}

fn encode_frame<'a>(env: Env<'a>, data: &[u8], meta: Term<'a>, encoder: Encoder) -> Term<'a> {
    let result = Metadata::from_term(meta).and_then(|meta| {
        meta.check_raw_len(data.len())?;
        error::catch_panic(|| encoder(data, &meta))
    });
    reply(env, result)
}

fn reply(env: Env<'_>, result: Result<Vec<u8>, CodecError>) -> Term<'_> {
    use rustler::Encoder as _;

    result
        .and_then(|bytes| {
            let mut binary = rustler::OwnedBinary::new(bytes.len())
                .ok_or_else(|| CodecError::Internal("out of memory".into()))?;
            binary.as_mut_slice().copy_from_slice(&bytes);
            Ok(binary.release(env))
        })
        .encode(env)
}

rustler::init!("Elixir.DicomCodecs.Native");
