defmodule WebWeb.Router do
  use WebWeb, :router

  pipeline :browser do
    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_live_flash
    plug :put_root_layout, html: {WebWeb.Layouts, :root}
    plug :protect_from_forgery
    plug :put_secure_browser_headers
  end

  pipeline :api do
    plug :accepts, ["json"]
  end

  scope "/", WebWeb do
    pipe_through :browser

    get "/", PageController, :home
    live "/dashboard", DashboardLive, :index
    live "/devices", DeviceLive, :index
  end

  scope "/api", WebWeb do
    pipe_through :api

    get "/health", ApiController, :health
  end
end
