defmodule DicomCodecs do
  @moduledoc """
  NIF-backed DICOM pixel data codecs for the `dicom` library.

  Each codec implements `Dicom.Codec` with Rust NIFs (Rustler) over these crates:

  - **JPEG Baseline/Extended/Lossless**: `jpeg-decoder` to decode, `jpeg-encoder`
    to encode (lossy Baseline only)
  - **JPEG 2000**: `openjp2`, OpenJPEG ported to Rust
  - **JPEG-LS**: `charls-sys`, bindings to CharLS

  ## Usage

  Add `dicom_codecs` to your dependencies. The application registers every
  codec with `Dicom.Codec.Registry` at startup, so a codec can be looked up by
  the Transfer Syntax UID of the data set:

      {:ok, codec} = Dicom.Codec.Registry.lookup(transfer_syntax_uid)
      {:ok, pixels} = codec.decode(compressed_frame, %{rows: 512, columns: 512, bits_allocated: 16})

  ## Frames and metadata

  Decoding returns one frame of native pixel data: samples interleaved by
  pixel (Planar Configuration 0), little-endian, `:bits_allocated / 8` bytes
  per sample. Encoding expects the same layout.

  `:rows` and `:columns` are required. `:bits_allocated` defaults to 8,
  `:bits_stored` to `:bits_allocated`, `:samples_per_pixel` to 1,
  `:pixel_representation` and `:planar_configuration` to 0, and
  `:photometric_interpretation` to `"MONOCHROME2"` (or `"RGB"` for three
  samples). A decoded frame must match the metadata in size, samples per pixel
  and bytes per sample.

  ## Errors

  Every NIF runs on a dirty CPU scheduler and returns `{:error, {reason, message}}`
  instead of raising, where `reason` is one of:

  - `:invalid_metadata`: the metadata is malformed or describes an impossible frame
  - `:metadata_mismatch`: the input or the decoded frame disagrees with the metadata
  - `:unsupported`: valid input this codec does not handle
  - `:decode_failed` / `:encode_failed`: the codec rejected the data
  - `:internal_error`: a bug inside the codec (including a caught panic)

  ## Supported Transfer Syntaxes

  | Transfer Syntax | UID | Codec |
  |-----------------|-----|-------|
  | JPEG Baseline (Process 1) | 1.2.840.10008.1.2.4.50 | JPEG |
  | JPEG Extended (Processes 2 & 4) | 1.2.840.10008.1.2.4.51 | JPEG |
  | JPEG Lossless, Non-Hierarchical (Process 14) | 1.2.840.10008.1.2.4.57 | JPEG |
  | JPEG Lossless, First-Order Prediction | 1.2.840.10008.1.2.4.70 | JPEG |
  | JPEG-LS Lossless | 1.2.840.10008.1.2.4.80 | JPEGLS |
  | JPEG-LS Near-Lossless | 1.2.840.10008.1.2.4.81 | JPEGLS |
  | JPEG 2000 Lossless Only | 1.2.840.10008.1.2.4.90 | JPEG2000 |
  | JPEG 2000 | 1.2.840.10008.1.2.4.91 | JPEG2000 |
  """

  @doc """
  Returns the list of all codec modules provided by this package.
  """
  @spec codecs() :: [module()]
  def codecs do
    [
      DicomCodecs.JPEG,
      DicomCodecs.JPEG2000,
      DicomCodecs.JPEGLS
    ]
  end

  @doc """
  Registers all codecs with `Dicom.Codec.Registry`.

  Called automatically at application startup. Can also be called manually
  if you need to re-register after a `Registry.reset/0`.
  """
  @spec register_all() :: :ok
  def register_all do
    if Code.ensure_loaded?(Dicom.Codec.Registry) do
      Enum.each(codecs(), &Dicom.Codec.Registry.register/1)
    end

    :ok
  end
end
