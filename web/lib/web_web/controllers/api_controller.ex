defmodule WebWeb.ApiController do
  use WebWeb, :controller

  def health(conn, _params) do
    json(conn, %{
      status: "ok",
      service: "AIS — Autonomous Intelligence Socket",
      version: "0.1.0",
      timestamp: DateTime.utc_now() |> DateTime.to_iso8601(),
    })
  end
end
