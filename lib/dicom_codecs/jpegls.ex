defmodule DicomCodecs.JPEGLS do
  @moduledoc """
  JPEG-LS codec for DICOM transfer syntaxes: Lossless and Near-Lossless.

  Backed by CharLS through the `charls-sys` crate. Decoding handles any
  interleave mode and returns colour frames interleaved by pixel. Encoding is
  always lossless, sample-interleaved for colour frames. JPEG-LS has no
  signed samples: signed frames are encoded as their two's-complement bit
  patterns, as other DICOM implementations do.

  Errors are `{:error, {reason, message}}` as described in `DicomCodecs`.
  """

  @behaviour Dicom.Codec

  alias DicomCodecs.{Metadata, Native}

  @transfer_syntaxes [
    # JPEG-LS Lossless Image Compression
    "1.2.840.10008.1.2.4.80",
    # JPEG-LS Lossy (Near-Lossless) Image Compression
    "1.2.840.10008.1.2.4.81"
  ]

  @impl true
  def transfer_syntax_uids, do: @transfer_syntaxes

  @impl true
  def decode(frame_binary, metadata) when is_binary(frame_binary) and is_map(metadata) do
    Native.jpegls_decode(frame_binary, Metadata.normalize(metadata))
  end

  @impl true
  def encode(raw_pixels, metadata) when is_binary(raw_pixels) and is_map(metadata) do
    Native.jpegls_encode(raw_pixels, Metadata.normalize(metadata))
  end
end
