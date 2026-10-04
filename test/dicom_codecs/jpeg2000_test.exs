defmodule DicomCodecs.JPEG2000Test do
  use ExUnit.Case, async: true

  alias DicomCodecs.JPEG2000
  alias DicomCodecs.Test.Pixels

  # Encoded by OpenJPEG's opj_compress (C), see test/fixtures/generate.exs.
  @interop [
    {"opj_8bit_mono.j2k", []},
    {"opj_8bit_rgb_mct.j2k", [samples_per_pixel: 3]},
    {"opj_16bit_mono.j2k", [bits_stored: 16]},
    {"opj_16bit_signed_mono.j2k", [bits_stored: 16, signed: true]}
  ]

  for {file, opts} <- @interop do
    @file_name file
    @opts opts

    test "decodes #{file} from OpenJPEG exactly" do
      j2k = Pixels.fixture!("jpeg2000/" <> @file_name)

      assert JPEG2000.decode(j2k, Pixels.metadata(40, 48, @opts)) ==
               {:ok, Pixels.frame(40, 48, @opts)}
    end
  end

  test "writes a raw J2K codestream" do
    metadata = Pixels.metadata(40, 48)

    assert {:ok, <<0xFF, 0x4F, 0xFF, 0x51, _::binary>>} =
             JPEG2000.encode(Pixels.frame(40, 48), metadata)
  end

  test "applies the reversible colour transform only for YBR_RCT" do
    opts = [samples_per_pixel: 3]
    pixels = Pixels.frame(40, 48, opts)
    rgb = Pixels.metadata(40, 48, opts)
    rct = Map.put(rgb, :photometric_interpretation, "YBR_RCT")

    assert {:ok, plain} = JPEG2000.encode(pixels, rgb)
    assert {:ok, transformed} = JPEG2000.encode(pixels, rct)
    assert mct_flag(plain) == 0
    assert mct_flag(transformed) == 1
    # Decoding inverts the transform: both give back the RGB frame.
    assert JPEG2000.decode(plain, rgb) == {:ok, pixels}
    assert JPEG2000.decode(transformed, rct) == {:ok, pixels}
  end

  # Multiple component transform byte of the COD marker segment (ISO 15444-1 A.6.1).
  defp mct_flag(codestream) do
    [{offset, _}] = :binary.matches(codestream, <<0xFF, 0x52>>) |> Enum.take(1)
    :binary.at(codestream, offset + 8)
  end
end
