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
      deps: deps(),
      description: "NIF-backed DICOM pixel data codecs (JPEG, JPEG 2000, JPEG-LS) for the dicom library",
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

  defp deps do
    [
      {:dicom, "~> 0.9", optional: true},
      {:rustler, "~> 0.36", runtime: false},
      {:rustler_precompiled, "~> 0.8"},
      {:ex_doc, "~> 0.34", only: :dev, runtime: false}
    ]
  end

  defp package do
    [
      licenses: ["MIT"],
      links: %{"GitHub" => @source_url},
      files: ~w(lib native .formatter.exs mix.exs README.md LICENSE CHANGELOG.md checksum-*.exs)
    ]
  end

  defp docs do
    [
      main: "readme",
      source_url: @source_url,
      extras: ["README.md", "CHANGELOG.md"]
    ]
  end

  defp aliases do
    []
  end
end
