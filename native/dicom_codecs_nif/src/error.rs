use rustler::{Encoder, Env, Term};
use std::panic::{self, AssertUnwindSafe};

rustler::atoms! {
    invalid_metadata,
    metadata_mismatch,
    unsupported,
    decode_failed,
    encode_failed,
    internal_error,
}

/// Failure reported to Elixir as `{reason, message}`.
#[derive(Debug)]
pub enum CodecError {
    /// The metadata map is malformed or describes an impossible frame.
    InvalidMetadata(String),
    /// The input is valid but disagrees with the metadata (size, dimensions, depth).
    MetadataMismatch(String),
    /// Valid input that this codec does not handle.
    Unsupported(String),
    DecodeFailed(String),
    EncodeFailed(String),
    /// A bug or resource exhaustion inside the codec.
    Internal(String),
}

impl Encoder for CodecError {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        let (reason, message) = match self {
            CodecError::InvalidMetadata(m) => (invalid_metadata(), m),
            CodecError::MetadataMismatch(m) => (metadata_mismatch(), m),
            CodecError::Unsupported(m) => (unsupported(), m),
            CodecError::DecodeFailed(m) => (decode_failed(), m),
            CodecError::EncodeFailed(m) => (encode_failed(), m),
            CodecError::Internal(m) => (internal_error(), m),
        };
        (reason, message.as_str()).encode(env)
    }
}

/// Runs a codec call, turning a panic into `CodecError::Internal` so that the
/// caller always gets an error tuple instead of an exception.
pub fn catch_panic<T>(f: impl FnOnce() -> Result<T, CodecError>) -> Result<T, CodecError> {
    panic::catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|payload| {
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic".into());
        Err(CodecError::Internal(format!("codec panicked: {message}")))
    })
}
