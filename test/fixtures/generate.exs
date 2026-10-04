# Regenerates the externally encoded fixtures used by the decoder tests.
#
#   mix run test/fixtures/generate.exs
#
# Needs `cjpeg` from libjpeg-turbo >= 3.0 (lossless JPEG) and `opj_compress`
# from OpenJPEG. The pixels come from DicomCodecs.Test.Pixels, so the tests
# recompute the expected frames instead of storing them.

Code.require_file("../support/pixels.ex", __DIR__)
alias DicomCodecs.Test.Pixels

defmodule Fixtures do
  def tmp(name), do: Path.join(System.tmp_dir!(), "dicom_codecs_" <> name)

  # Binary PGM/PPM: 16-bit samples are big-endian (Netpbm).
  def pnm(rows, columns, samples_per_pixel, bits) do
    magic = if samples_per_pixel == 3, do: "P6", else: "P5"
    max = Bitwise.bsl(1, bits) - 1
    size = if max > 255, do: 16, else: 8

    body =
      for v <- Pixels.values(rows, columns, samples_per_pixel, bits),
          into: <<>>,
          do: <<v::big-size(size)>>

    path = tmp("#{rows}x#{columns}x#{samples_per_pixel}_#{bits}.pnm")
    File.write!(path, "#{magic}\n#{columns} #{rows}\n#{max}\n" <> body)
    path
  end

  def run!(cmd, args) do
    {out, status} = System.cmd(cmd, args, stderr_to_stdout: true)
    if status != 0, do: raise("#{cmd} #{Enum.join(args, " ")} failed:\n#{out}")
  end
end

dir = __DIR__
{rows, columns} = {40, 48}

# JPEG lossless (process 14): {file, samples per pixel, bits, predictor}
for {file, spp, bits, predictor} <- [
      {"lossless_8bit_mono_p1.jpg", 1, 8, 1},
      {"lossless_8bit_rgb_p1.jpg", 3, 8, 1},
      {"lossless_12bit_mono_p6.jpg", 1, 12, 6},
      {"lossless_16bit_mono_p1.jpg", 1, 16, 1},
      {"lossless_16bit_rgb_p7.jpg", 3, 16, 7}
    ] do
  input = Fixtures.pnm(rows, columns, spp, bits)
  color = if spp == 3, do: ["-rgb"], else: ["-grayscale"]
  out = Path.join([dir, "jpeg", file])
  File.mkdir_p!(Path.dirname(out))

  Fixtures.run!(
    "cjpeg",
    color ++ ["-precision", "#{bits}", "-lossless", "#{predictor}", "-outfile", out, input]
  )
end

# JPEG 2000 from OpenJPEG's own encoder (lossless; MCT on for RGB by default)
for {file, spp, bits} <- [
      {"opj_8bit_mono.j2k", 1, 8},
      {"opj_8bit_rgb_mct.j2k", 3, 8},
      {"opj_16bit_mono.j2k", 1, 16}
    ] do
  input = Fixtures.pnm(rows, columns, spp, bits)
  out = Path.join([dir, "jpeg2000", file])
  File.mkdir_p!(Path.dirname(out))
  Fixtures.run!("opj_compress", ["-i", input, "-o", out])
end

# Signed 16-bit through a little-endian raw input
raw = Fixtures.tmp("signed16.rawl")
File.write!(raw, Pixels.frame(rows, columns, bits_stored: 16, signed: true))

Fixtures.run!("opj_compress", [
  "-i",
  raw,
  "-o",
  Path.join([dir, "jpeg2000", "opj_16bit_signed_mono.j2k"]),
  "-F",
  "#{columns},#{rows},1,16,s"
])
