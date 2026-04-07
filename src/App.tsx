import { Show, onMount } from "solid-js";
import "./App.css";
import { TabBar } from "./components/TabBar";
import { TimelineGrid } from "./components/TimelineGrid";
import { Favourites } from "./components/Favourites";
import { WeekView } from "./components/WeekView";
import { Settings } from "./components/Settings";
import { StaleBanner } from "./components/StaleBanner";
import { ErrorState } from "./components/ErrorState";
import { activeTab, settingsOpen, refreshData, channels, hoveredProgram } from "./lib/state";
import { setLocale, locale } from "./i18n";
import { formatTime } from "./lib/time";
import { api } from "./lib/api";
import type { Locale } from "./lib/types";

export default function App() {
  onMount(async () => {
    const saved = await api.getSetting("locale");
    if (saved) setLocale(saved as Locale);
    refreshData().catch((e: unknown) => console.error("Initial fetch failed:", e));
  });

  return (
    <div class="app">
      <TabBar />
      <StaleBanner />
      <div class="app-content">
        <Show when={channels.error}>
          <ErrorState />
        </Show>
        <Show when={!channels.error}>
          <Show when={activeTab() === "nu"}>
            <TimelineGrid />
          </Show>
          <Show when={activeTab() === "favoriter"}>
            <Favourites />
          </Show>
          <Show when={activeTab() === "vecka"}>
            <WeekView />
          </Show>
        </Show>
      </div>
      <Show when={settingsOpen()}>
        <Settings />
      </Show>
      <Show when={hoveredProgram()}>
        {(data) => {
          const pos = () => {
            const d = data();
            const x = Math.min(d.x + 12, window.innerWidth - 320);
            const y = d.y < window.innerHeight - 100 ? d.y + 16 : d.y - 60;
            return `left:${x}px;top:${y}px`;
          };
          return (
            <div class="program-tooltip" style={pos()}>
              <div class="tooltip-title">{data().program.title}</div>
              <div class="tooltip-time">
                {formatTime(data().program.start_time, locale())} – {formatTime(data().program.end_time, locale())}
              </div>
            </div>
          );
        }}
      </Show>
    </div>
  );
}
