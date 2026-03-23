defmodule DicomCodecsTest do
  use ExUnit.Case

  describe "codecs/0" do
    test "returns all codec modules" do
      codecs = DicomCodecs.codecs()
      assert DicomCodecs.JPEG in codecs
      assert DicomCodecs.JPEG2000 in codecs
      assert DicomCodecs.JPEGLS in codecs
      assert length(codecs) == 3
    end
  end

  describe "transfer syntax coverage" do
    test "JPEG covers 4 transfer syntaxes" do
      uids = DicomCodecs.JPEG.transfer_syntax_uids()
      assert "1.2.840.10008.1.2.4.50" in uids
      assert "1.2.840.10008.1.2.4.51" in uids
      assert "1.2.840.10008.1.2.4.57" in uids
      assert "1.2.840.10008.1.2.4.70" in uids
    end

    test "JPEG 2000 covers 2 transfer syntaxes" do
      uids = DicomCodecs.JPEG2000.transfer_syntax_uids()
      assert "1.2.840.10008.1.2.4.90" in uids
      assert "1.2.840.10008.1.2.4.91" in uids
    end

    test "JPEG-LS covers 2 transfer syntaxes" do
      uids = DicomCodecs.JPEGLS.transfer_syntax_uids()
      assert "1.2.840.10008.1.2.4.80" in uids
      assert "1.2.840.10008.1.2.4.81" in uids
    end

    test "all codecs together cover 8 transfer syntaxes" do
      all_uids =
        DicomCodecs.codecs()
        |> Enum.flat_map(& &1.transfer_syntax_uids())

      assert length(all_uids) == 8
      assert length(Enum.uniq(all_uids)) == 8
    end
  end
end
