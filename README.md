# DicomCodecs

NIF-backed pixel data codecs for the [`dicom`](https://hex.pm/packages/dicom) library. Adds hardware-accelerated JPEG, JPEG 2000, and JPEG-LS decompression to DICOM file handling.

## Installation

```elixir
def deps do
  [
    {:dicom, "~> 0.9"},
    {:dicom_codecs, "~> 0.1"}
  ]
end
```

That's it. Codecs auto-register with `Dicom.Codec.Registry` at application startup. Then use `Dicom.PixelData` as usual:

```elixir
{:ok, ds} = Dicom.parse_file("ct_scan.dcm")

# Compressed frames are transparently decompressed
{:ok, raw_pixels} = Dicom.PixelData.decode_frame(ds, 0)
{:ok, all_frames} = Dicom.PixelData.decode_all_frames(ds)
```

## Supported Transfer Syntaxes

| Transfer Syntax | UID | Backend |
|---|---|---|
| JPEG Baseline | `1.2.840.10008.1.2.4.50` | jpeg-decoder/jpeg-encoder |
| JPEG Extended | `1.2.840.10008.1.2.4.51` | jpeg-decoder |
| JPEG Lossless (Process 14) | `1.2.840.10008.1.2.4.57` | jpeg-decoder |
| JPEG Lossless (First-Order Pred.) | `1.2.840.10008.1.2.4.70` | jpeg-decoder |
| JPEG-LS Lossless | `1.2.840.10008.1.2.4.80` | CharLS |
| JPEG-LS Near-Lossless | `1.2.840.10008.1.2.4.81` | CharLS |
| JPEG 2000 Lossless Only | `1.2.840.10008.1.2.4.90` | OpenJPEG |
| JPEG 2000 | `1.2.840.10008.1.2.4.91` | OpenJPEG |
| RLE Lossless | `1.2.840.10008.1.2.5` | Built-in (pure Elixir, in `dicom`) |

## Building from source

Precompiled binaries are provided for common platforms. To force a source build:

```bash
DICOM_CODECS_BUILD=1 mix deps.compile dicom_codecs
```

Requires Rust 1.70+ toolchain (`rustup` recommended).

## License

MIT
