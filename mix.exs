defmodule DicomCodecs.MixProject do
  use Mix.Project

  @version "0.1.0"
  @source_url "https://github.com/Balneario-de-Cofrentes/dicom_codecs"

  def project do
    [
      app: :dicom_codecs,
      version: @version,
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      elixirc_paths: elixirc_paths(Mix.env()),
      deps: deps(),
      description:
        "NIF-backed DICOM pixel data codecs (JPEG, JPEG 2000, JPEG-LS) for the dicom library",
      package: package(),
      docs: docs(),
      aliases: aliases()
    ]
  end

  def application do
    [
      extra_applications: [:logger],
      mod: {DicomCodecs.Application, []}
    ]
  end

  defp elixirc_paths(:test), do: ["lib", "test/support"]
  defp elixirc_paths(_), do: ["lib"]

  defp deps do
    [
      {:dicom, "~> 0.9", optional: true},
      # Needed only to build the NIF from source; see DicomCodecs.Native.
      {:rustler, "~> 0.38", optional: true},
      {:rustler_precompiled, "~> 0.10"},
      {:ex_doc, "~> 0.34", only: :dev, runtime: false}
    ]
  end

  # `mix test` also runs the crate's unit tests.
  defp aliases do
    [
      test: [
        "cmd cargo test --release --quiet --manifest-path native/dicom_codecs_nif/Cargo.toml",
        "test"
      ]
    ]
  end

  defp package do
    [
      licenses: ["MIT"],
      links: %{"GitHub" => @source_url},
      files:
        ~w(lib native/dicom_codecs_nif/src native/dicom_codecs_nif/Cargo.toml native/dicom_codecs_nif/Cargo.lock .formatter.exs mix.exs README.md LICENSE THIRD_PARTY_NOTICES.md CHANGELOG.md checksum-*.exs)
    ]
  end

  defp docs do
    [
      main: "readme",
      source_url: @source_url,
      extras: ["README.md", "CHANGELOG.md", "THIRD_PARTY_NOTICES.md"]
    ]
  end
end
