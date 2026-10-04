defmodule DicomCodecs.JPEGTest do
  use ExUnit.Case, async: true

  alias DicomCodecs.JPEG
  alias DicomCodecs.Test.Pixels

  # Encoded by libjpeg-turbo's cjpeg, see test/fixtures/generate.exs.
  @lossless [
    {"lossless_8bit_mono_p1.jpg", []},
    {"lossless_8bit_rgb_p1.jpg", [samples_per_pixel: 3]},
    {"lossless_12bit_mono_p6.jpg", [bits_stored: 12]},
    {"lossless_16bit_mono_p1.jpg", [bits_stored: 16]},
    {"lossless_16bit_rgb_p7.jpg", [bits_stored: 16, samples_per_pixel: 3]}
  ]

  for {file, opts} <- @lossless do
    @file_name file
    @opts opts

    test "decodes #{file} exactly" do
      jpeg = Pixels.fixture!("jpeg/" <> @file_name)

      assert JPEG.decode(jpeg, Pixels.metadata(40, 48, @opts)) ==
               {:ok, Pixels.frame(40, 48, @opts)}
    end
  end

  for {name, opts} <- [{"monochrome", []}, {"RGB", [samples_per_pixel: 3]}] do
    @opts opts

    test "baseline round trip of 8-bit #{name} stays close to the original" do
      pixels = Pixels.frame(40, 48, @opts)
      metadata = Pixels.metadata(40, 48, @opts)

      assert {:ok, <<0xFF, 0xD8, _::binary>> = jpeg} = JPEG.encode(pixels, metadata)
      assert {:ok, decoded} = JPEG.decode(jpeg, metadata)
      assert byte_size(decoded) == byte_size(pixels)

      errors =
        Enum.zip_with(:binary.bin_to_list(decoded), :binary.bin_to_list(pixels), &abs(&1 - &2))

      # Quality 95 with 4:4:4 chroma on noisy data.
      assert Enum.sum(errors) / length(errors) < 2
      assert Enum.max(errors) <= 16
    end
  end

  test "baseline encoding rejects 16-bit frames" do
    opts = [bits_stored: 12]

    assert {:error, {:unsupported, message}} =
             JPEG.encode(Pixels.frame(40, 48, opts), Pixels.metadata(40, 48, opts))

    assert message =~ "8-bit"
  end
end
