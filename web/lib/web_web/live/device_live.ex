defmodule WebWeb.DeviceLive do
  use WebWeb, :live_view

  @impl true
  def mount(_params, _session, socket) do
    socket =
      socket
      |> assign(:page_title, "Devices — AIS")
      |> assign(:devices, [])
      |> assign(:selected_device, nil)

    {:ok, socket}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="text-2xl font-semibold text-gray-900">Device Registry</h1>
      <p class="mt-1 text-sm text-gray-500">
        Browse and manage registered AIS devices
      </p>

      <div class="mt-8 rounded-lg bg-white p-6 shadow">
        <p class="text-sm text-gray-500">
          Connect to <code class="rounded bg-gray-100 px-1 py-0.5 text-xs">googuly.online/aisocket</code>
          to see live registered devices, or start a device on your local network.
        </p>

        <div class="mt-4 rounded-md bg-blue-50 p-4">
          <div class="flex">
            <div class="text-sm text-blue-700">
              <p class="font-medium">Quick Start</p>
              <pre class="mt-2 text-xs"><code>curl -X POST https://googuly.online/aisocket/register.php \
  -H "Content-Type: application/json" \
  -d '{"name":"my_device","ip":"192.168.1.100","port":8442,"prompt":"Test device"}'</code></pre>
            </div>
          </div>
        </div>
      </div>
    </div>
    """
  end
end
