defmodule DicomCodecs.Application do
  @moduledoc false
  use Application

  @impl true
  def start(_type, _args) do
    DicomCodecs.register_all()
    Supervisor.start_link([], strategy: :one_for_one, name: DicomCodecs.Supervisor)
  end
end
