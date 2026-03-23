defmodule DicomCodecs.JPEGLS do
  @moduledoc """
  JPEG-LS codec for DICOM transfer syntaxes.

  Supports JPEG-LS Lossless and Near-Lossless.
  Backed by Rust NIF using the `charls` crate.
  """

  @behaviour Dicom.Codec

  @transfer_syntaxes [
    # JPEG-LS Lossless Image Compression
    "1.2.840.10008.1.2.4.80",
    # JPEG-LS Lossy (Near-Lossless) Image Compression
    "1.2.840.10008.1.2.4.81"
  ]

  @impl true
  def transfer_syntax_uids, do: @transfer_syntaxes

  @impl true
  def decode(frame_binary, metadata) when is_binary(frame_binary) do
    DicomCodecs.Native.jpegls_decode(frame_binary, normalize_metadata(metadata))
  end

  @impl true
  def encode(raw_pixels, metadata) when is_binary(raw_pixels) do
    DicomCodecs.Native.jpegls_encode(raw_pixels, normalize_metadata(metadata))
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
