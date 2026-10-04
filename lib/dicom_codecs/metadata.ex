defmodule DicomCodecs.Metadata do
  @moduledoc false
  # Fills the optional `Dicom.Codec.metadata()` keys the NIFs need. Validation
  # happens in the NIF, which reports `{:error, {:invalid_metadata, message}}`.

  @spec normalize(map()) :: map()
  def normalize(metadata) when is_map(metadata) do
    bits_allocated = Map.get(metadata, :bits_allocated, 8)
    samples_per_pixel = Map.get(metadata, :samples_per_pixel, 1)

    %{
      rows: Map.get(metadata, :rows),
      columns: Map.get(metadata, :columns),
      bits_allocated: bits_allocated,
      bits_stored: Map.get(metadata, :bits_stored, bits_allocated),
      samples_per_pixel: samples_per_pixel,
      photometric_interpretation:
        Map.get(metadata, :photometric_interpretation, default_photometric(samples_per_pixel)),
      pixel_representation: Map.get(metadata, :pixel_representation, 0),
      planar_configuration: Map.get(metadata, :planar_configuration, 0)
    }
  end

  defp default_photometric(3), do: "RGB"
  defp default_photometric(_), do: "MONOCHROME2"
end
