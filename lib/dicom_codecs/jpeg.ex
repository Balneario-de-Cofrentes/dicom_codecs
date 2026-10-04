defmodule DicomCodecs.JPEG do
  @moduledoc """
  JPEG codec for DICOM transfer syntaxes: Baseline (Process 1), Extended
  (Processes 2 & 4) and Lossless (Process 14, any predictor, including
  First-Order Prediction).

  Decoding uses the `jpeg-decoder` crate: 8-bit lossy streams and lossless
  streams of 2 to 16 bits. Three-component lossy frames come back converted
  to RGB; lossless frames come back as stored. 12-bit lossy streams (Extended
  Process 4) are not supported and return `{:error, {:decode_failed, _}}`.

  Encoding uses the `jpeg-encoder` crate and always produces lossy Baseline
  JPEG at quality 95 with 4:4:4 chroma, so it only accepts 8-bit frames.
  Lossless JPEG encoding is not available; use JPEG-LS or JPEG 2000.

  Errors are `{:error, {reason, message}}` as described in `DicomCodecs`.
  """

  @behaviour Dicom.Codec

  alias DicomCodecs.{Metadata, Native}

  @transfer_syntaxes [
    # JPEG Baseline (Process 1): Default Transfer Syntax for Lossy JPEG 8 Bit
    "1.2.840.10008.1.2.4.50",
    # JPEG Extended (Processes 2 & 4)
    "1.2.840.10008.1.2.4.51",
    # JPEG Lossless, Non-Hierarchical (Process 14)
    "1.2.840.10008.1.2.4.57",
    # JPEG Lossless, Non-Hierarchical, First-Order Prediction
    "1.2.840.10008.1.2.4.70"
  ]

  @impl true
  def transfer_syntax_uids, do: @transfer_syntaxes

  @impl true
  def decode(frame_binary, metadata) when is_binary(frame_binary) and is_map(metadata) do
    Native.jpeg_decode(frame_binary, Metadata.normalize(metadata))
  end

  @impl true
  def encode(raw_pixels, metadata) when is_binary(raw_pixels) and is_map(metadata) do
    Native.jpeg_encode(raw_pixels, Metadata.normalize(metadata))
  end
end
