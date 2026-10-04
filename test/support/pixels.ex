defmodule DicomCodecs.Test.Pixels do
  @moduledoc false
  # Deterministic test frames: a diagonal gradient plus pseudo-random noise, so
  # that codecs see real structure, every bit of the sample range, and both
  # extremes (the first pixel is the minimum, the last one the maximum).

  import Bitwise

  @doc """
  Native little-endian frame interleaved by pixel. With `signed: true` the
  values span the two's-complement range of `bits_stored`.
  """
  def frame(rows, columns, opts \\ []) do
    samples_per_pixel = Keyword.get(opts, :samples_per_pixel, 1)
    bits_stored = Keyword.get(opts, :bits_stored, 8)
    bits_allocated = Keyword.get(opts, :bits_allocated, if(bits_stored > 8, do: 16, else: 8))
    signed? = Keyword.get(opts, :signed, false)

    for value <- values(rows, columns, samples_per_pixel, bits_stored, signed?), into: <<>> do
      <<value::little-signed-size(bits_allocated)>>
    end
  end

  @doc "Metadata map matching `frame/3` with the same options."
  def metadata(rows, columns, opts \\ []) do
    bits_stored = Keyword.get(opts, :bits_stored, 8)

    %{
      rows: rows,
      columns: columns,
      samples_per_pixel: Keyword.get(opts, :samples_per_pixel, 1),
      bits_stored: bits_stored,
      bits_allocated: Keyword.get(opts, :bits_allocated, if(bits_stored > 8, do: 16, else: 8)),
      pixel_representation: if(Keyword.get(opts, :signed, false), do: 1, else: 0)
    }
    |> Map.merge(Map.new(Keyword.take(opts, [:photometric_interpretation])))
  end

  @doc "Contents of a file under test/fixtures."
  def fixture!(path), do: File.read!(Path.join([__DIR__, "..", "fixtures", path]))

  @doc "Sample values in raster order, as plain integers."
  def values(rows, columns, samples_per_pixel, bits_stored, signed? \\ false) do
    max = (1 <<< bits_stored) - 1
    last = rows * columns * samples_per_pixel - 1
    offset = if signed?, do: 1 <<< (bits_stored - 1), else: 0

    for i <- 0..last do
      pixel = div(i, samples_per_pixel)
      {y, x} = {div(pixel, columns), rem(pixel, columns)}

      value =
        cond do
          i < samples_per_pixel -> 0
          i > last - samples_per_pixel -> max
          true -> rem(div(max * (x + y), rows + columns) + noise(i, max), max + 1)
        end

      value - offset
    end
  end

  # Small LCG-derived noise, a few percent of the range.
  defp noise(i, max), do: rem(i * 1_103_515_245 + 12_345 &&& 0x7FFFFFFF, div(max, 16) + 1)
end
