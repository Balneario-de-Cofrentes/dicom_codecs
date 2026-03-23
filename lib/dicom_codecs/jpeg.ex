defmodule DicomCodecs.JPEG do
  @moduledoc """
  JPEG codec for DICOM transfer syntaxes.

  Supports JPEG Baseline (Process 1), Extended (Processes 2 & 4),
  and Lossless (Process 14, First-Order Prediction).

  Backed by Rust NIF using the `image` crate for decode and
  `mozjpeg`/`jpeg-encoder` for encode.
  """

  @behaviour Dicom.Codec

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
  def decode(frame_binary, metadata) when is_binary(frame_binary) do
    DicomCodecs.Native.jpeg_decode(frame_binary, normalize_metadata(metadata))
  end

  @impl true
  def encode(raw_pixels, metadata) when is_binary(raw_pixels) do
    DicomCodecs.Native.jpeg_encode(raw_pixels, normalize_metadata(metadata))
  end

  defp normalize_metadata(metadata) do
    %{
      rows: Map.get(metadata, :rows, 0),
      columns: Map.get(metadata, :columns, 0),
      bits_allocated: Map.get(metadata, :bits_allocated, 8),
      bits_stored: Map.get(metadata, :bits_stored, 8),
      samples_per_pixel: Map.get(metadata, :samples_per_pixel, 1),
      photometric_interpretation: Map.get(metadata, :photometric_interpretation, "MONOCHROME2"),
      pixel_representation: Map.get(metadata, :pixel_representation, 0)
    }
  end
end
