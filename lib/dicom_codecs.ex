defmodule DicomCodecs do
  @moduledoc """
  NIF-backed DICOM pixel data codecs for the `dicom` library.

  Provides hardware-accelerated decompression and compression for common
  DICOM transfer syntaxes via Rust NIFs wrapping established codec libraries:

  - **JPEG Baseline/Extended/Lossless** — via `image` crate (libjpeg-turbo compatible)
  - **JPEG 2000** — via `jpeg2k` crate (OpenJPEG bindings)
  - **JPEG-LS** — via `charls` crate (CharLS bindings)

  ## Usage

  Add `dicom_codecs` to your dependencies and codecs are auto-registered
  with `Dicom.Codec.Registry` at application startup:

      # mix.exs
      {:dicom_codecs, "~> 0.1"}

  Then use `Dicom.PixelData.decode_frame/2` or `decode_all_frames/1`
  as usual — compressed frames are transparently decompressed.

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
