defmodule DicomCodecs.LosslessTest do
  use ExUnit.Case, async: true

  alias DicomCodecs.Test.Pixels

  # {name, rows, columns, frame options}
  @shapes [
    {"8-bit monochrome", 40, 48, []},
    {"8-bit RGB", 40, 48, [samples_per_pixel: 3]},
    {"12-bit monochrome in 16 bits", 40, 48, [bits_stored: 12]},
    {"16-bit monochrome", 40, 48, [bits_stored: 16]},
    {"16-bit RGB", 40, 48, [bits_stored: 16, samples_per_pixel: 3]},
    {"16-bit signed monochrome", 40, 48, [bits_stored: 16, signed: true]},
    {"single pixel", 1, 1, [bits_stored: 16]},
    {"single row", 1, 257, []},
    {"odd sizes", 7, 13, [samples_per_pixel: 3]}
  ]

  for codec <- [DicomCodecs.JPEG2000, DicomCodecs.JPEGLS],
      {name, rows, columns, opts} <- @shapes do
    @codec codec
    @shape {rows, columns, opts}

    test "#{inspect(codec)} round-trips #{name} byte for byte" do
      {rows, columns, opts} = @shape
      pixels = Pixels.frame(rows, columns, opts)
      metadata = Pixels.metadata(rows, columns, opts)

      assert {:ok, encoded} = @codec.encode(pixels, metadata)
      assert byte_size(encoded) > 0
      assert @codec.decode(encoded, metadata) == {:ok, pixels}
    end
  end

  test "JPEG 2000 round-trips signed 12-bit samples sign-extended to 16 bits" do
    opts = [bits_stored: 12, signed: true]
    pixels = Pixels.frame(40, 48, opts)
    metadata = Pixels.metadata(40, 48, opts)

    assert {:ok, encoded} = DicomCodecs.JPEG2000.encode(pixels, metadata)
    assert DicomCodecs.JPEG2000.decode(encoded, metadata) == {:ok, pixels}
  end

  test "bits above bits_stored are ignored when encoding" do
    opts = [bits_stored: 12]
    pixels = Pixels.frame(40, 48, opts)
    metadata = Pixels.metadata(40, 48, opts)
    # Set the four unused high bits of every sample.
    dirty = for <<v::little-16 <- pixels>>, into: <<>>, do: <<Bitwise.bor(v, 0xF000)::little-16>>

    for codec <- [DicomCodecs.JPEG2000, DicomCodecs.JPEGLS] do
      assert {:ok, encoded} = codec.encode(dirty, metadata)
      assert codec.decode(encoded, metadata) == {:ok, pixels}, inspect(codec)
    end
  end
end
