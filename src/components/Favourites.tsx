import { For, Show, createSignal, createMemo } from "solid-js";
import { t, locale } from "../i18n";
import { favouritePrograms, favourites, toggleFavourite, channels } from "../lib/state";
import { formatTime, relativeDay } from "../lib/time";
import type { Program, Channel } from "../lib/types";

export function Favourites() {
  const [confirmTitle, setConfirmTitle] = createSignal<string | null>(null);

  const grouped = createMemo(() => {
    const progs = favouritePrograms() ?? [];
    const map = new Map<string, Program[]>();
    for (const p of progs) {
      const list = map.get(p.title) ?? [];
      list.push(p);
      map.set(p.title, list);
    }
    return map;
  });

  const channelName = (id: string): string => {
    const ch = (channels() ?? []).find((c: Channel) => c.id === id);
    return ch?.name ?? id;
  };

  const handleRemove = async (title: string) => {
    await toggleFavourite(title);
    setConfirmTitle(null);
  };

  return (
    <div class="favourites-view">
      <Show
        when={(favourites() ?? []).length > 0}
        fallback={
          <p style="text-align:center;color:var(--text-secondary);padding:40px">
            {t("no_favourites")}
          </p>
        }
      >
        <For each={[...grouped().entries()]}>
          {([title, airings]) => (
            <div class="fav-group">
              <div class="fav-group-header">
                <span>{"\u2605"} {title}</span>
                <button
                  class="star-btn active"
                  onClick={() => setConfirmTitle(title)}
                  style="font-size:13px;padding:4px 10px"
                >
                  {"\u2715"}
                </button>
              </div>
              <For each={airings}>
                {(p: Program) => (
                  <div class="fav-airing">
                    <span class="channel">{channelName(p.channel_id)}</span>
                    <span class="day">{relativeDay(p.start_time, locale())}</span>
                    <span>
                      {formatTime(p.start_time, locale())} – {formatTime(p.end_time, locale())}
                    </span>
                  </div>
                )}
              </For>
            </div>
          )}
        </For>
      </Show>

      <Show when={confirmTitle()}>
        <div class="confirm-overlay" onClick={() => setConfirmTitle(null)}>
          <div class="confirm-dialog" onClick={(e: MouseEvent) => e.stopPropagation()}>
            <p>{t("remove_favourite")}</p>
            <p style="font-weight:600;margin-bottom:16px">{confirmTitle()}</p>
            <div class="actions">
              <button onClick={() => setConfirmTitle(null)}>{t("cancel")}</button>
              <button class="primary" onClick={() => handleRemove(confirmTitle()!)}>
                {t("confirm")}
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
