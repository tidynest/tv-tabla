import { For, Show, createSignal, createResource, createMemo } from "solid-js";
import { t, locale } from "../i18n";
import { selectedWeek, setSelectedWeek, channels } from "../lib/state";
import { api } from "../lib/api";
import { weekNumber, weekStartDate, startOfDay, endOfDay, formatTime } from "../lib/time";
import type { Program, Channel } from "../lib/types";

const MAX_WEEKS = 4;

export function WeekView() {
  const [expandedCell, setExpandedCell] = createSignal<string | null>(null);

  const weekStart = createMemo(() => weekStartDate(selectedWeek()));

  const days = createMemo(() => {
    const start = weekStart();
    return Array.from({ length: 7 }, (_, i) => {
      const d = new Date(start);
      d.setDate(start.getDate() + i);
      return d;
    });
  });

  const weekLabel = createMemo(() => {
    const start = weekStart();
    const end = new Date(start);
    end.setDate(start.getDate() + 6);
    const wn = weekNumber(start);
    const fmt = (d: Date) =>
      d.toLocaleDateString(locale(), { day: "numeric", month: "short" });
    return `${t("week")} ${wn} (${fmt(start)}\u2013${fmt(end)})`;
  });

  const [weekPrograms] = createResource(
    () => ({ week: selectedWeek() }),
    async () => {
      const start = weekStartDate(selectedWeek());
      const end = new Date(start);
      end.setDate(start.getDate() + 7);
      return api.getPrograms(startOfDay(start), endOfDay(end));
    }
  );

  const dayNames = createMemo(() =>
    days().map((d) =>
      d.toLocaleDateString(locale(), { weekday: "short", day: "numeric" })
    )
  );

  function cellPrograms(channelId: string, dayIndex: number): Program[] {
    const day = days()[dayIndex];
    const dayS = startOfDay(day);
    const dayE = endOfDay(day);
    return (weekPrograms() ?? []).filter(
      (p: Program) =>
        p.channel_id === channelId && p.start_time >= dayS && p.start_time < dayE
    );
  }

  function cellKey(channelId: string, dayIndex: number): string {
    return `${channelId}_${dayIndex}`;
  }

  function displayPrograms(channelId: string, dayIndex: number): Program[] {
    const all = cellPrograms(channelId, dayIndex);
    if (expandedCell() === cellKey(channelId, dayIndex)) return all;
    const day = days()[dayIndex];
    const ptStart = new Date(day); ptStart.setHours(18, 0, 0, 0);
    const ptEnd = new Date(day); ptEnd.setHours(23, 0, 0, 0);
    const ptS = Math.floor(ptStart.getTime() / 1000);
    const ptE = Math.floor(ptEnd.getTime() / 1000);
    const primetime = all.filter((p: Program) => p.start_time >= ptS && p.start_time < ptE);
    return primetime.length > 0 ? primetime.slice(0, 4) : all.slice(0, 3);
  }

  return (
    <div class="week-view">
      <div class="week-nav">
        <button onClick={() => setSelectedWeek((w) => w - 1)} disabled={selectedWeek() <= 0}>
          &#9664;
        </button>
        <span>{weekLabel()}</span>
        <button onClick={() => setSelectedWeek((w) => w + 1)} disabled={selectedWeek() >= MAX_WEEKS}>
          &#9654;
        </button>
      </div>

      <div class="week-grid">
        <div class="day-header" style="border-right:1px solid var(--border)">&nbsp;</div>
        <For each={dayNames()}>
          {(name: string) => <div class="day-header">{name}</div>}
        </For>

        <For each={channels() ?? []}>
          {(ch: Channel) => (
            <>
              <div class="channel-label" style="border-bottom:1px solid var(--border);border-right:1px solid var(--border)">
                {ch.name}
              </div>
              <For each={[0, 1, 2, 3, 4, 5, 6]}>
                {(dayIdx: number) => (
                  <div
                    class="week-cell"
                    onClick={() =>
                      setExpandedCell(
                        expandedCell() === cellKey(ch.id, dayIdx) ? null : cellKey(ch.id, dayIdx)
                      )
                    }
                  >
                    <For each={displayPrograms(ch.id, dayIdx)}>
                      {(p: Program) => (
                        <div style="margin-bottom:2px">
                          <span style="color:var(--accent)">{formatTime(p.start_time, locale())}</span>{" "}
                          {p.title}
                        </div>
                      )}
                    </For>
                    <Show when={cellPrograms(ch.id, dayIdx).length > displayPrograms(ch.id, dayIdx).length}>
                      <div style="color:var(--text-secondary);font-style:italic">
                        +{cellPrograms(ch.id, dayIdx).length - displayPrograms(ch.id, dayIdx).length} ...
                      </div>
                    </Show>
                  </div>
                )}
              </For>
            </>
          )}
        </For>
      </div>
    </div>
  );
}
