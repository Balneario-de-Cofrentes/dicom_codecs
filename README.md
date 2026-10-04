# DicomCodecs

JPEG, JPEG 2000 and JPEG-LS pixel data codecs for the [`dicom`](https://hex.pm/packages/dicom) library, as Rust NIFs built with [Rustler](https://github.com/rusterlium/rustler). Each codec implements `Dicom.Codec`.

## Installation

```elixir
def deps do
  [
    {:dicom, "~> 0.9"},
    {:dicom_codecs, "~> 0.1"},
    # Only to build from source (see below):
    # {:rustler, "~> 0.38"}
  ]
end
```

The application registers every codec with `Dicom.Codec.Registry` at startup. `dicom` 0.9 does not call codecs itself: look one up by the Transfer Syntax UID and decode the frames from `Dicom.PixelData`:

```elixir
{:ok, ds} = Dicom.parse_file("ct_scan.dcm")

uid = Dicom.DataSet.decoded_value(ds, {0x0002, 0x0010})
{:ok, codec} = Dicom.Codec.Registry.lookup(uid)
{:ok, frame} = Dicom.PixelData.frame(ds, 0)

metadata = %{
  rows: Dicom.DataSet.decoded_value(ds, {0x0028, 0x0010}),
  columns: Dicom.DataSet.decoded_value(ds, {0x0028, 0x0011}),
  bits_allocated: Dicom.DataSet.decoded_value(ds, {0x0028, 0x0100}),
  bits_stored: Dicom.DataSet.decoded_value(ds, {0x0028, 0x0101}),
  samples_per_pixel: Dicom.DataSet.decoded_value(ds, {0x0028, 0x0002}),
  pixel_representation: Dicom.DataSet.decoded_value(ds, {0x0028, 0x0103})
}

{:ok, pixels} = codec.decode(frame, metadata)
```

Decoded frames are native pixel data: little-endian, interleaved by pixel, `bits_allocated / 8` bytes per sample. Failures come back as `{:error, {reason, message}}`; the reasons are listed in the `DicomCodecs` module docs.

## Supported Transfer Syntaxes

| Transfer Syntax | UID | Decode | Encode |
|---|---|---|---|
| JPEG Baseline | `1.2.840.10008.1.2.4.50` | `jpeg-decoder` | `jpeg-encoder`, quality 95 |
| JPEG Extended | `1.2.840.10008.1.2.4.51` | `jpeg-decoder`, 8-bit only | Baseline output |
| JPEG Lossless (Process 14) | `1.2.840.10008.1.2.4.57` | `jpeg-decoder`, 2–16 bits | no (Baseline output) |
| JPEG Lossless (First-Order Pred.) | `1.2.840.10008.1.2.4.70` | `jpeg-decoder`, 2–16 bits | no (Baseline output) |
| JPEG-LS Lossless | `1.2.840.10008.1.2.4.80` | CharLS | CharLS, lossless |
| JPEG-LS Near-Lossless | `1.2.840.10008.1.2.4.81` | CharLS | lossless output |
| JPEG 2000 Lossless Only | `1.2.840.10008.1.2.4.90` | `openjp2` | `openjp2`, lossless |
| JPEG 2000 | `1.2.840.10008.1.2.4.91` | `openjp2` | lossless output |
| RLE Lossless | `1.2.840.10008.1.2.5` | built into `dicom` | built into `dicom` |

The `encode/2` callback of `Dicom.Codec` does not receive the Transfer Syntax, so each codec has one encoder: the JPEG codec always writes 8-bit Baseline, the others always write lossless streams. `openjp2` is OpenJPEG ported to Rust; CharLS is C++ and links statically through `charls-sys`.

## Not supported

- Lossless JPEG encoding: the JPEG codec only writes 8-bit Baseline.
- 12-bit lossy JPEG (Extended, Process 4) decoding.
- Separate colour planes (Planar Configuration 1) and subsampled JPEG 2000 components.
- Signed JPEG-LS samples narrower than Bits Allocated come back as stored bit patterns, without sign extension.

The first three return `{:error, {:unsupported | :decode_failed, message}}`. The crate does not build for 32-bit or big-endian hosts.

## Scheduling

Decoding or encoding a frame takes milliseconds to seconds, far over the 1 ms a NIF may hold a normal scheduler, so every NIF runs on a dirty CPU scheduler. `test/dicom_codecs/native_test.exs` checks it with microstate accounting.

## Precompiled NIFs and building from source

Releases ship precompiled NIFs (NIF 2.15, which loads on OTP 22 and later) for `aarch64-apple-darwin`, `x86_64-apple-darwin`, `aarch64-unknown-linux-gnu` and `x86_64-unknown-linux-gnu` (glibc 2.35 or later). Set `DICOM_CODECS_BUILD=1` to compile from source on other targets (musl, Windows) or to skip the download. Building needs `{:rustler, "~> 0.38"}` in your deps, a Rust toolchain (tested with 1.94 and 1.99), CMake and a C++ compiler (for CharLS). Only 64-bit little-endian targets are supported.

## Tests

```bash
mix test
```

runs the crate's unit tests and the ExUnit suite. In this repo's dev and test environments the NIF is always compiled from source, so tests exercise the working copy, not a downloaded binary. The JPEG lossless and part of the JPEG 2000 fixtures come from independent encoders (libjpeg-turbo's `cjpeg`, OpenJPEG's `opj_compress`); `mix run test/fixtures/generate.exs` rebuilds them.

## Releasing

1. Bump `@version` in `mix.exs` and the `CHANGELOG.md`, commit, and push a `v<version>` tag. The `Precompiled NIFs` workflow builds every target and attaches the archives to the GitHub release.
2. `mix rustler_precompiled.download DicomCodecs.Native --all --print` writes `checksum-Elixir.DicomCodecs.Native.exs`. Commit it: with that file present, installs download the binaries instead of compiling.
3. `mix hex.publish`.

## License

MIT. The NIF statically links CharLS (BSD-3-Clause), OpenJPEG through `openjp2` (BSD-2-Clause) and code from the Independent JPEG Group; their notices are in `THIRD_PARTY_NOTICES.md`.
