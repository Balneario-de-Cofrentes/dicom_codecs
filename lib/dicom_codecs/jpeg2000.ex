defmodule DicomCodecs.JPEG2000 do
  @moduledoc """
  JPEG 2000 codec for DICOM transfer syntaxes: Lossless Only and JPEG 2000
  (lossless or lossy).

  Backed by the `openjp2` crate, OpenJPEG ported to Rust. Decoding accepts
  J2K codestreams and JP2 files of 1 to 16 bits, signed or unsigned, and
  inverts any colour transform. Encoding writes a lossless J2K codestream
  (reversible 5/3 wavelet). The reversible colour transform is applied only
  when `:photometric_interpretation` is `"YBR_RCT"`, so RGB frames stay RGB.

  Errors are `{:error, {reason, message}}` as described in `DicomCodecs`.
  """

  @behaviour Dicom.Codec

  alias DicomCodecs.{Metadata, Native}

  @transfer_syntaxes [
    # JPEG 2000 Image Compression (Lossless Only)
    "1.2.840.10008.1.2.4.90",
    # JPEG 2000 Image Compression
    "1.2.840.10008.1.2.4.91"
  ]

  @impl true
  def transfer_syntax_uids, do: @transfer_syntaxes

  @impl true
  def decode(frame_binary, metadata) when is_binary(frame_binary) and is_map(metadata) do
    Native.jpeg2000_decode(frame_binary, Metadata.normalize(metadata))
  end

  @impl true
  def encode(raw_pixels, metadata) when is_binary(raw_pixels) and is_map(metadata) do
    Native.jpeg2000_encode(raw_pixels, Metadata.normalize(metadata))
  end
end
