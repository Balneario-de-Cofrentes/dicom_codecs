defmodule DicomCodecs.Native do
  @moduledoc false
  # Rustler NIF bridge. Functions are replaced at load time by native implementations.
  #
  # Every NIF runs on a dirty CPU scheduler and returns `{:ok, binary}` or
  # `{:error, {reason, message}}`.
  #
  # Precompiled NIFs are downloaded only when the package ships the checksum file
  # that `mix rustler_precompiled.download` writes at release time. Without it, or
  # with DICOM_CODECS_BUILD=1, the crate is compiled from source.

  mix_config = Mix.Project.config()
  version = mix_config[:version]
  github_url = mix_config[:package][:links]["GitHub"]

  checksum_file = Path.expand("../../checksum-#{inspect(__MODULE__)}.exs", __DIR__)
  @external_resource checksum_file

  use RustlerPrecompiled,
    otp_app: :dicom_codecs,
    crate: "dicom_codecs_nif",
    base_url: "#{github_url}/releases/download/v#{version}",
    force_build:
      System.get_env("DICOM_CODECS_BUILD") in ["1", "true"] or not File.exists?(checksum_file),
    # Keep in sync with .github/workflows/release.yml. Other targets need DICOM_CODECS_BUILD=1.
    targets: ~w(
      aarch64-apple-darwin
      x86_64-apple-darwin
      aarch64-unknown-linux-gnu
      x86_64-unknown-linux-gnu
    ),
    version: version

  def jpeg_decode(_frame, _metadata), do: :erlang.nif_error(:nif_not_loaded)
  def jpeg_encode(_pixels, _metadata), do: :erlang.nif_error(:nif_not_loaded)

  def jpeg2000_decode(_frame, _metadata), do: :erlang.nif_error(:nif_not_loaded)
  def jpeg2000_encode(_pixels, _metadata), do: :erlang.nif_error(:nif_not_loaded)

  def jpegls_decode(_frame, _metadata), do: :erlang.nif_error(:nif_not_loaded)
  def jpegls_encode(_pixels, _metadata), do: :erlang.nif_error(:nif_not_loaded)
end
