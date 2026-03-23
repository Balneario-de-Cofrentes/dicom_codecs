defmodule DicomCodecs.Native do
  @moduledoc false
  # Rustler NIF bridge. Functions are replaced at load time by native implementations.

  mix_config = Mix.Project.config()
  version = mix_config[:version]
  github_url = mix_config[:package][:links]["GitHub"] || ""

  use RustlerPrecompiled,
    otp_app: :dicom_codecs,
    crate: "dicom_codecs_nif",
    base_url: "#{github_url}/releases/download/v#{version}",
    force_build: System.get_env("DICOM_CODECS_BUILD") in ["1", "true"],
    version: version

  # JPEG decode/encode
  def jpeg_decode(_binary, _metadata), do: :erlang.nif_error(:nif_not_loaded)
  def jpeg_encode(_binary, _metadata), do: :erlang.nif_error(:nif_not_loaded)

  # JPEG 2000 decode/encode
  def jpeg2000_decode(_binary, _metadata), do: :erlang.nif_error(:nif_not_loaded)
  def jpeg2000_encode(_binary, _metadata), do: :erlang.nif_error(:nif_not_loaded)

  # JPEG-LS decode/encode
  def jpegls_decode(_binary, _metadata), do: :erlang.nif_error(:nif_not_loaded)
  def jpegls_encode(_binary, _metadata), do: :erlang.nif_error(:nif_not_loaded)
end
