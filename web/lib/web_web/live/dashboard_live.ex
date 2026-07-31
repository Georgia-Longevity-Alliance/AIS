defmodule WebWeb.DashboardLive do
  use WebWeb, :live_view

  @impl true
  def mount(_params, _session, socket) do
    socket =
      socket
      |> assign(:page_title, "AIS Dashboard")
      |> assign(:devices, mock_devices())
      |> assign(:recent_traces, mock_traces())

    {:ok, socket}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="px-4 py-6 sm:px-6 lg:px-8">
      <h1 class="text-2xl font-semibold text-gray-900">AIS Dashboard</h1>
      <p class="mt-1 text-sm text-gray-500">
        Autonomous Intelligence Socket — device overview
      </p>

      <!-- Stats -->
      <dl class="mt-5 grid grid-cols-1 gap-5 sm:grid-cols-3">
        <div class="overflow-hidden rounded-lg bg-white px-4 py-5 shadow sm:p-6">
          <dt class="truncate text-sm font-medium text-gray-500">Active Devices</dt>
          <dd class="mt-1 text-3xl font-semibold tracking-tight text-gray-900">
            <%= length(@devices) %>
          </dd>
        </div>
        <div class="overflow-hidden rounded-lg bg-white px-4 py-5 shadow sm:p-6">
          <dt class="truncate text-sm font-medium text-gray-500">Recent Traces</dt>
          <dd class="mt-1 text-3xl font-semibold tracking-tight text-gray-900">
            <%= length(@recent_traces) %>
          </dd>
        </div>
        <div class="overflow-hidden rounded-lg bg-white px-4 py-5 shadow sm:p-6">
          <dt class="truncate text-sm font-medium text-gray-500">System Status</dt>
          <dd class="mt-1 text-3xl font-semibold tracking-tight text-green-600">
            Online
          </dd>
        </div>
      </dl>

      <!-- Devices Table -->
      <h2 class="mt-8 text-lg font-medium text-gray-900">Registered Devices</h2>
      <div class="mt-4 flow-root">
        <div class="-mx-4 -my-2 overflow-x-auto sm:-mx-6 lg:-mx-8">
          <div class="inline-block min-w-full py-2 align-middle sm:px-6 lg:px-8">
            <table class="min-w-full divide-y divide-gray-300">
              <thead>
                <tr>
                  <th class="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900">Device</th>
                  <th class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">Platform</th>
                  <th class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">Risk Class</th>
                  <th class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">Status</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-200">
                <%= for device <- @devices do %>
                  <tr>
                    <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-gray-900">
                      <%= device.name %>
                    </td>
                    <td class="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                      <%= device.platform %>
                    </td>
                    <td class="whitespace-nowrap px-3 py-4 text-sm">
                      <span class={"inline-flex rounded-full px-2 text-xs font-semibold leading-5 #{risk_color(device.risk)}"}>
                        <%= device.risk %>
                      </span>
                    </td>
                    <td class="whitespace-nowrap px-3 py-4 text-sm">
                      <span class="inline-flex rounded-full bg-green-100 px-2 text-xs font-semibold leading-5 text-green-800">
                        online
                      </span>
                    </td>
                  </tr>
                <% end %>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <!-- Recent Traces -->
      <h2 class="mt-8 text-lg font-medium text-gray-900">Recent Intervention Traces</h2>
      <div class="mt-4 space-y-4">
        <%= for trace <- @recent_traces do %>
          <div class="rounded-lg bg-white p-4 shadow">
            <div class="flex items-center justify-between">
              <h3 class="text-sm font-medium text-gray-900"><%= trace.device %></h3>
              <span class={[
                "inline-flex rounded-full px-2 text-xs font-semibold leading-5",
                outcome_color(trace.outcome)
              ]}>
                <%= trace.outcome %>
              </span>
            </div>
            <p class="mt-1 text-sm text-gray-500"><%= trace.diagnosis %></p>
            <p class="mt-1 text-xs text-gray-400">Confidence: <%= trace.confidence %>%</p>
          </div>
        <% end %>
      </div>
    </div>
    """
  end

  defp mock_devices do
    [
      %{name: "ARGUS-OS1 V6", platform: "Jetson Orin NX", risk: "medium"},
      %{name: "Mower V1", platform: "ESP32", risk: "low"},
      %{name: "Lab Incubator", platform: "Raspberry Pi", risk: "medium"},
    ]
  end

  defp mock_traces do
    [
      %{
        device: "ARGUS-OS1 V6",
        diagnosis: "Photobleaching detected — reduced 488nm power by 30%",
        confidence: 85,
        outcome: "resolved",
      },
      %{
        device: "Mower V1",
        diagnosis: "Obstacle in path — rerouting around garden bed",
        confidence: 92,
        outcome: "resolved",
      },
      %{
        device: "Lab Incubator",
        diagnosis: "Temperature drift +0.3°C — recalibrating PID",
        confidence: 78,
        outcome: "mitigated",
      },
    ]
  end

  defp risk_color("low"), do: "bg-green-100 text-green-800"
  defp risk_color("medium"), do: "bg-yellow-100 text-yellow-800"
  defp risk_color("high"), do: "bg-red-100 text-red-800"
  defp risk_color(_), do: "bg-gray-100 text-gray-800"

  defp outcome_color("resolved"), do: "bg-green-100 text-green-800"
  defp outcome_color("mitigated"), do: "bg-yellow-100 text-yellow-800"
  defp outcome_color("escalated"), do: "bg-red-100 text-red-800"
  defp outcome_color(_), do: "bg-gray-100 text-gray-800"
end
