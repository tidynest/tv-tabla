import { For, Show, createMemo } from "solid-js";
import { t, locale } from "../i18n";
import {
  channels,
  programs,
  currentTime,
  dayExpanded,
  setDayExpanded,
  setPopupProgram,
  popupProgram,
} from "../lib/state";
import { formatTime } from "../lib/time";
import { ProgramPopup } from "./ProgramPopup";
import type { Program, Channel } from "../lib/types";

const PIXELS_PER_MINUTE = 4;
const VISIBLE_HOURS = 3;

export function TimelineGrid() {
  const timeRange = createMemo(() => {
    const now = currentTime();
    const startMin = Math.floor(now / 1800) * 1800;
    const hours = dayExpanded() ? 24 : VISIBLE_HOURS;
    return { start: startMin, end: startMin + hours * 3600 };
  });

  const totalWidth = createMemo(() => {
    const { start, end } = timeRange();
    return ((end - start) / 60) * PIXELS_PER_MINUTE;
  });

  const timeSlots = createMemo(() => {
    const { start, end } = timeRange();
    const slots: number[] = [];
    for (let t = start; t < end; t += 1800) {
      slots.push(t);
    }
    return slots;
  });

  const nowOffset = createMemo(() => {
    const { start } = timeRange();
    return ((currentTime() - start) / 60) * PIXELS_PER_MINUTE;
  });

  const programsByChannel = createMemo(() => {
    const map = new Map<string, Program[]>();
    for (const p of programs() ?? []) {
      const list = map.get(p.channel_id) ?? [];
      list.push(p);
      map.set(p.channel_id, list);
    }
    return map;
  });

  function blockStyle(program: Program): string {
    const { start } = timeRange();
    const left = Math.max(0, ((program.start_time - start) / 60) * PIXELS_PER_MINUTE);
    const right = ((program.end_time - start) / 60) * PIXELS_PER_MINUTE;
    const width = Math.max(48, right - left);
    return `left:${left}px;width:${width}px`;
  }

  let popupAnchor = { x: 0, y: 0 };

  function handleBlockClick(program: Program, e: MouseEvent) {
    popupAnchor = { x: e.clientX, y: e.clientY };
    setPopupProgram(program);
  }

  return (
    <div>
      <div class="timeline-grid">
        <div class="channel-labels">
          <div class="channel-label" style="height:32px;font-size:12px;color:var(--text-secondary)">
            &nbsp;
          </div>
          <For each={channels() ?? []}>
            {(ch: Channel) => <div class="channel-label">{ch.name}</div>}
          </For>
        </div>

        <div class="timeline-scroll">
          <div style={`width:${totalWidth()}px;position:relative`}>
            <div class="time-header">
              <For each={timeSlots()}>
                {(ts: number) => (
                  <span style={`width:${30 * PIXELS_PER_MINUTE}px`}>
                    {formatTime(ts, locale())}
                  </span>
                )}
              </For>
            </div>

            <For each={channels() ?? []}>
              {(ch: Channel) => (
                <div class="timeline-row">
                  <For each={programsByChannel().get(ch.id) ?? []}>
                    {(p: Program) => (
                      <div
                        class="program-block"
                        style={blockStyle(p)}
                        onClick={(e) => handleBlockClick(p, e)}
                        title={p.title}
                      >
                        {p.title}
                      </div>
                    )}
                  </For>
                </div>
              )}
            </For>

            <div class="now-marker" style={`left:${nowOffset()}px;top:0;height:100%`} />
          </div>
        </div>
      </div>

      <button class="expand-btn" onClick={() => setDayExpanded(!dayExpanded())}>
        {dayExpanded() ? t("show_less") : t("show_more")} {dayExpanded() ? "\u25B2" : "\u25BC"}
      </button>

      <Show when={popupProgram()}>
        <ProgramPopup
          program={popupProgram()!}
          x={popupAnchor.x}
          y={popupAnchor.y}
          onClose={() => setPopupProgram(null)}
        />
      </Show>
    </div>
  );
}
