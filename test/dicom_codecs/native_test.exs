defmodule DicomCodecs.NativeTest do
  # Synchronous: microstate accounting is VM-wide, so no other test may run NIFs
  # while it is on.
  use ExUnit.Case, async: false

  alias DicomCodecs.{Metadata, Native}

  @side 1024

  # A NIF that runs longer than 1 ms must run on a dirty scheduler. Microstate
  # accounting splits each scheduler thread's time by type; if the NIF ran on a
  # normal scheduler the dirty CPU threads would stay idle during the call.
  for {codec, bits} <- [jpeg: 8, jpeg2000: 16, jpegls: 16] do
    @encode :"#{codec}_encode"
    @decode :"#{codec}_decode"
    @bits bits

    test "#{codec} encode and decode run on dirty CPU schedulers" do
      metadata = Metadata.normalize(%{rows: @side, columns: @side, bits_allocated: @bits})
      :rand.seed(:exsss, {4, 5, 6})
      pixels = :rand.bytes(@side * @side * div(@bits, 8))

      {{:ok, encoded}, encode} = measure(fn -> apply(Native, @encode, [pixels, metadata]) end)
      {{:ok, _decoded}, decode} = measure(fn -> apply(Native, @decode, [encoded, metadata]) end)

      for {name, %{wall: wall, dirty_cpu: dirty_cpu}} <- [{@encode, encode}, {@decode, decode}] do
        assert wall > 2_000, "#{name} took #{wall} µs, too short to tell schedulers apart"

        assert dirty_cpu >= 0.8 * wall,
               "#{name}: #{dirty_cpu} µs on dirty CPU schedulers out of #{wall} µs"
      end
    end
  end

  defp measure(fun) do
    :erlang.system_flag(:microstate_accounting, :reset)
    :erlang.system_flag(:microstate_accounting, true)
    {wall, result} = :timer.tc(fun)
    :erlang.system_flag(:microstate_accounting, false)

    dirty_cpu =
      for %{type: :dirty_cpu_scheduler, counters: counters} <-
            :erlang.statistics(:microstate_accounting),
          reduce: 0 do
        acc -> acc + :erlang.convert_time_unit(counters.emulator, :perf_counter, :microsecond)
      end

    {result, %{wall: wall, dirty_cpu: dirty_cpu}}
  end
end
