//! JPEG 2000 through `openjp2`, the Rust port of OpenJPEG.
//!
//! openjp2 only reads and writes files or caller-supplied callbacks, so this
//! module bridges in-memory buffers through its C-style stream API.

use crate::error::CodecError;
use crate::frame::{self, Image, Metadata};
use openjp2::openjpeg::{
    opj_stream_set_read_function, opj_stream_set_seek_function, opj_stream_set_skip_function,
    opj_stream_set_user_data, opj_stream_set_user_data_length, opj_stream_set_write_function,
};
use openjp2::{
    opj_cparameters_t, opj_dparameters_t, opj_image, opj_image_comptparm, opj_stream_t, Codec,
    J2KFormat, Stream, OPJ_CLRSPC_GRAY, OPJ_CLRSPC_SRGB, OPJ_CODEC_J2K, OPJ_CODEC_JP2,
};
use std::ffi::{c_char, c_void, CStr};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

const STREAM_BUFFER: usize = 64 * 1024;
/// OpenJPEG's default number of DWT resolution levels.
const MAX_RESOLUTIONS: u32 = 6;

/// Decodes a J2K codestream or JP2 file. Lossless and lossy streams both
/// decode; colour transforms (RCT/ICT) are inverted by OpenJPEG.
pub fn decode(data: &[u8], meta: &Metadata) -> Result<Image, CodecError> {
    let failed = |m: &str| CodecError::DecodeFailed(format!("JPEG 2000: {m}"));

    let format = match openjp2::detect_format(data).map_err(|e| failed(&e))? {
        J2KFormat::JP2 => OPJ_CODEC_JP2,
        J2KFormat::J2K => OPJ_CODEC_J2K,
        J2KFormat::JPT => return Err(CodecError::Unsupported("JPEG 2000: JPT streams".into())),
    };
    let mut codec = Codec::new_decoder(format).ok_or_else(|| failed("cannot create decoder"))?;
    let mut messages = Messages::attach(&mut codec);

    let mut source = Cursor::new(data);
    let mut stream = reader_stream(&mut source, data.len());
    let mut params = opj_dparameters_t::default();

    if codec.setup_decoder(&mut params) == 0 {
        return Err(failed(&messages.take("cannot set up decoder")));
    }
    let mut image = codec
        .read_header(&mut stream)
        .ok_or_else(|| failed(&messages.take("invalid header")))?;
    // Check before decoding allocates: a corrupt header can declare any size.
    let (width, height, components, bytes_per_sample) = geometry(&image)?;
    meta.check_header(width, height, components, bytes_per_sample)?;
    if codec.decode(&mut stream, &mut image) == 0 || codec.end_decompress(&mut stream) == 0 {
        return Err(failed(&messages.take("corrupt codestream")));
    }

    interleave(&image)
}

/// Encodes a frame as a lossless J2K codestream (reversible 5/3 wavelet, one
/// quality layer), the form DICOM encapsulates. The reversible colour
/// transform is applied only when the photometric interpretation is YBR_RCT,
/// so RGB frames stay RGB.
pub fn encode(raw: &[u8], meta: &Metadata) -> Result<Vec<u8>, CodecError> {
    let failed = |m: &str| CodecError::EncodeFailed(format!("JPEG 2000: {m}"));

    let mut image = build_image(raw, meta).ok_or_else(|| failed("cannot allocate image"))?;
    let mut params = lossless_parameters(meta);

    let mut codec =
        Codec::new_encoder(OPJ_CODEC_J2K).ok_or_else(|| failed("cannot create encoder"))?;
    let mut messages = Messages::attach(&mut codec);
    if codec.setup_encoder(&mut params, &mut image) == 0 {
        return Err(failed(&messages.take("invalid encoder parameters")));
    }

    let mut sink = Cursor::new(Vec::new());
    {
        let mut stream = writer_stream(&mut sink);
        let encoded = codec.start_compress(&mut image, &mut stream) != 0
            && codec.encode(&mut stream) != 0
            && codec.end_compress(&mut stream) != 0;
        if !encoded {
            return Err(failed(&messages.take("encoding failed")));
        }
        // Dropping the stream flushes its buffer into `sink`.
    }
    Ok(sink.into_inner())
}

fn lossless_parameters(meta: &Metadata) -> opj_cparameters_t {
    // Each resolution level halves the image; OpenJPEG rejects levels that
    // would shrink the smallest side below one pixel.
    let smallest_side = meta.rows.min(meta.columns);
    opj_cparameters_t {
        // One layer at rate 0 (the default tcp_rates) means no truncation: lossless.
        tcp_numlayers: 1,
        cp_disto_alloc: 1,
        irreversible: 0,
        tcp_mct: (meta.photometric_interpretation == "YBR_RCT") as c_char,
        numresolution: MAX_RESOLUTIONS.min(smallest_side.ilog2() + 1) as i32,
        ..Default::default()
    }
}

fn build_image(raw: &[u8], meta: &Metadata) -> Option<Box<opj_image>> {
    let components = meta.samples_per_pixel as usize;
    let param = opj_image_comptparm {
        dx: 1,
        dy: 1,
        w: meta.columns,
        h: meta.rows,
        x0: 0,
        y0: 0,
        prec: meta.bits_stored,
        bpp: meta.bits_stored,
        sgnd: meta.pixel_representation,
    };
    let color_space = if components == 1 {
        OPJ_CLRSPC_GRAY
    } else {
        OPJ_CLRSPC_SRGB
    };
    let mut image = opj_image::create(&vec![param; components], color_space)?;
    image.x1 = meta.columns;
    image.y1 = meta.rows;

    let samples = meta.samples(raw);
    for (c, plane) in image.comps_data_mut_iter()?.enumerate() {
        for (dst, src) in plane
            .iter_mut()
            .zip(samples.iter().skip(c).step_by(components))
        {
            *dst = *src;
        }
    }
    Some(image)
}

/// Width, height, components and bytes per sample of an image whose
/// components all share one size and precision, the only layout DICOM uses.
fn geometry(image: &opj_image) -> Result<(u32, u32, u32, usize), CodecError> {
    let unsupported = |m: String| Err(CodecError::Unsupported(format!("JPEG 2000: {m}")));

    let comps = match image.comps() {
        Some(comps) if !comps.is_empty() => comps,
        _ => return Err(CodecError::DecodeFailed("JPEG 2000: no components".into())),
    };
    let first = &comps[0];
    if comps.iter().any(|c| c.dx != 1 || c.dy != 1) {
        return unsupported("subsampled components".into());
    }
    if comps
        .iter()
        .any(|c| (c.w, c.h, c.prec, c.sgnd) != (first.w, first.h, first.prec, first.sgnd))
    {
        return unsupported("components with different sizes or precisions".into());
    }
    let bytes_per_sample = match first.prec {
        1..=8 => 1,
        9..=16 => 2,
        p => return unsupported(format!("{p}-bit samples")),
    };
    Ok((first.w, first.h, comps.len() as u32, bytes_per_sample))
}

/// Converts OpenJPEG's one-plane-per-component layout into interleaved
/// little-endian samples.
fn interleave(image: &opj_image) -> Result<Image, CodecError> {
    let (width, height, components, bytes_per_sample) = geometry(image)?;
    let planes = image
        .comps_data_iter()
        .into_iter()
        .flatten()
        .map(|c| c.data)
        .collect::<Vec<_>>();
    if planes.len() != components as usize {
        return Err(CodecError::DecodeFailed(
            "JPEG 2000: missing component data".into(),
        ));
    }
    let pixels = width as usize * height as usize;
    let samples = (0..pixels).flat_map(|i| planes.iter().map(move |plane| plane[i]));

    Ok(Image {
        width,
        height,
        components,
        bytes_per_sample,
        data: frame::samples_to_bytes(samples, bytes_per_sample),
    })
}

/// Collects OpenJPEG error messages so failures carry its explanation.
/// The codec keeps a raw pointer to the Vec, hence the Box: it must not move.
#[allow(clippy::box_collection)]
struct Messages(Box<Vec<String>>);

impl Messages {
    fn attach(codec: &mut Codec) -> Self {
        let mut messages = Messages(Box::default());
        let sink = &mut *messages.0 as *mut Vec<String> as *mut c_void;
        codec.set_error_handler(Some(collect_message), sink);
        messages
    }

    fn take(&mut self, fallback: &str) -> String {
        if self.0.is_empty() {
            fallback.to_string()
        } else {
            std::mem::take(&mut *self.0).join("; ")
        }
    }
}

unsafe extern "C" fn collect_message(msg: *const c_char, sink: *mut c_void) {
    if msg.is_null() || sink.is_null() {
        return;
    }
    let messages = &mut *(sink as *mut Vec<String>);
    messages.push(CStr::from_ptr(msg).to_string_lossy().trim_end().to_string());
}

// -- In-memory streams ---------------------------------------------------------
//
// The stream receives a raw pointer to a cursor owned by the caller, which must
// outlive the stream. No free function is registered: the caller keeps ownership.

fn reader_stream<R: Read + Seek>(source: &mut R, len: usize) -> Stream {
    let mut stream = Stream::new_custom(STREAM_BUFFER, true);
    let raw = &mut stream as *mut Stream as *mut opj_stream_t;
    unsafe {
        opj_stream_set_user_data(raw, source as *mut R as *mut c_void, None);
        opj_stream_set_user_data_length(raw, len as u64);
        opj_stream_set_read_function(raw, Some(read_cb::<R>));
        opj_stream_set_skip_function(raw, Some(skip_cb::<R>));
        opj_stream_set_seek_function(raw, Some(seek_cb::<R>));
    }
    stream
}

fn writer_stream<W: Write + Seek>(sink: &mut W) -> Stream {
    let mut stream = Stream::new_custom(STREAM_BUFFER, false);
    let raw = &mut stream as *mut Stream as *mut opj_stream_t;
    unsafe {
        opj_stream_set_user_data(raw, sink as *mut W as *mut c_void, None);
        opj_stream_set_write_function(raw, Some(write_cb::<W>));
        opj_stream_set_skip_function(raw, Some(skip_cb::<W>));
        opj_stream_set_seek_function(raw, Some(seek_cb::<W>));
    }
    stream
}

/// OpenJPEG signals end of stream and errors with `(size_t)-1`.
const STREAM_END: usize = usize::MAX;

unsafe extern "C" fn read_cb<R: Read>(buffer: *mut c_void, len: usize, data: *mut c_void) -> usize {
    let source = &mut *(data as *mut R);
    let buffer = std::slice::from_raw_parts_mut(buffer as *mut u8, len);
    match source.read(buffer) {
        Ok(0) | Err(_) => STREAM_END,
        Ok(n) => n,
    }
}

unsafe extern "C" fn write_cb<W: Write>(
    buffer: *mut c_void,
    len: usize,
    data: *mut c_void,
) -> usize {
    let sink = &mut *(data as *mut W);
    let buffer = std::slice::from_raw_parts(buffer as *const u8, len);
    match sink.write_all(buffer) {
        Ok(()) => len,
        Err(_) => STREAM_END,
    }
}

unsafe extern "C" fn skip_cb<S: Seek>(offset: i64, data: *mut c_void) -> i64 {
    let stream = &mut *(data as *mut S);
    match stream.seek(SeekFrom::Current(offset)) {
        Ok(_) => offset,
        Err(_) => -1,
    }
}

unsafe extern "C" fn seek_cb<S: Seek>(offset: i64, data: *mut c_void) -> i32 {
    let stream = &mut *(data as *mut S);
    match u64::try_from(offset).map(|o| stream.seek(SeekFrom::Start(o))) {
        Ok(Ok(_)) => 1,
        _ => 0,
    }
}
