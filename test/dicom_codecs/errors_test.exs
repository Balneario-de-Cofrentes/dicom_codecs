defmodule DicomCodecs.ErrorsTest do
  use ExUnit.Case, async: true

  alias DicomCodecs.Test.Pixels

  @codecs [DicomCodecs.JPEG, DicomCodecs.JPEG2000, DicomCodecs.JPEGLS]
  @metadata Pixels.metadata(40, 48)
  @pixels Pixels.frame(40, 48)

  @invalid_metadata [
    {"missing rows", Map.delete(@metadata, :rows)},
    {"negative rows", %{@metadata | rows: -1}},
    {"rows as a string", %{@metadata | rows: "40"}},
    {"zero columns", %{@metadata | columns: 0}},
    {"columns beyond 16 bits", %{@metadata | columns: 70_000}},
    {"two samples per pixel", %{@metadata | samples_per_pixel: 2}},
    {"12 bits allocated", %{@metadata | bits_allocated: 12}},
    {"zero bits stored", %{@metadata | bits_stored: 0}},
    {"bits stored above bits allocated", %{@metadata | bits_stored: 9}},
    {"pixel representation 2", %{@metadata | pixel_representation: 2}},
    {"photometric interpretation as an atom",
     Map.put(@metadata, :photometric_interpretation, :monochrome2)}
  ]

  for codec <- @codecs do
    @codec codec

    describe inspect(codec) do
      for {name, metadata} <- @invalid_metadata do
        @bad_metadata metadata

        test "rejects metadata with #{name}" do
          {:ok, encoded} = @codec.encode(@pixels, @metadata)

          assert {:error, {:invalid_metadata, message}} = @codec.encode(@pixels, @bad_metadata)
          assert is_binary(message)
          assert {:error, {:invalid_metadata, _}} = @codec.decode(encoded, @bad_metadata)
        end
      end

      test "rejects separate colour planes" do
        rgb = Map.merge(@metadata, %{samples_per_pixel: 3, planar_configuration: 1})

        assert {:error, {:unsupported, _}} =
                 @codec.encode(Pixels.frame(40, 48, samples_per_pixel: 3), rgb)
      end

      test "rejects pixel data whose size disagrees with the metadata" do
        short = binary_part(@pixels, 0, byte_size(@pixels) - 1)

        assert {:error, {:metadata_mismatch, message}} = @codec.encode(short, @metadata)
        assert message =~ "1919 bytes"
      end

      test "rejects a frame that decodes to other dimensions than the metadata" do
        {:ok, encoded} = @codec.encode(@pixels, @metadata)

        for wrong <- [%{@metadata | rows: 48, columns: 40}, %{@metadata | samples_per_pixel: 3}] do
          assert {:error, {:metadata_mismatch, message}} = @codec.decode(encoded, wrong)
          assert message =~ "48x40"
        end
      end

      test "rejects a frame whose sample width disagrees with bits_allocated" do
        {:ok, encoded} = @codec.encode(@pixels, @metadata)
        wide = %{@metadata | bits_allocated: 16}

        assert {:error, {:metadata_mismatch, _}} = @codec.decode(encoded, wide)
      end

      test "returns an error for data that is not a compressed frame" do
        for garbage <- [<<>>, <<0>>, :binary.copy(<<0xFF>>, 64), @pixels] do
          assert {:error, {reason, _}} = @codec.decode(garbage, @metadata)
          assert reason == :decode_failed
        end
      end

      test "returns an error for every truncation of a valid frame" do
        {:ok, encoded} = @codec.encode(@pixels, @metadata)
        # Every cut except the last two bytes, which only hold the end marker
        # some decoders do not require.
        for length <- 0..(byte_size(encoded) - 3) do
          {elapsed, result} =
            :timer.tc(fn -> @codec.decode(binary_part(encoded, 0, length), @metadata) end)

          assert {:error, _} = result, "truncated to #{length} of #{byte_size(encoded)} bytes"
          # CharLS alone spins ~2.5 s on a truncated stream; see jpegls.rs.
          assert elapsed < 100_000, "truncated to #{length} bytes took #{elapsed} µs"
        end
      end

      test "survives randomly corrupted frames" do
        {:ok, encoded} = @codec.encode(@pixels, @metadata)
        :rand.seed(:exsss, {1, 2, 3})

        for _ <- 1..300 do
          corrupted = corrupt(encoded)
          # Entropy-coded data has no checksum, so a flipped byte can still
          # decode; what matters is a result instead of a crash.
          assert match?({:ok, bin} when is_binary(bin), @codec.decode(corrupted, @metadata)) or
                   match?(
                     {:error, {reason, msg}} when is_atom(reason) and is_binary(msg),
                     @codec.decode(corrupted, @metadata)
                   )
        end
      end
    end
  end

  test "raises FunctionClauseError for non-binary or non-map arguments" do
    for codec <- @codecs do
      assert_raise FunctionClauseError, fn -> codec.decode(:frame, @metadata) end
      assert_raise FunctionClauseError, fn -> codec.encode(@pixels, rows: 40) end
    end
  end

  # Overwrites 1 to 4 random bytes.
  defp corrupt(binary) do
    Enum.reduce(1..:rand.uniform(4), binary, fn _, acc ->
      at = :rand.uniform(byte_size(acc)) - 1
      <<head::binary-size(at), _, tail::binary>> = acc
      <<head::binary, :rand.uniform(256) - 1, tail::binary>>
    end)
  end
end
