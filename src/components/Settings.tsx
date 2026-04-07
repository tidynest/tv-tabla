import { For } from "solid-js";
import { t, locale, setLocale, localeNames } from "../i18n";
import { allChannels, setSettingsOpen, toggleChannelVisibility } from "../lib/state";
import { api } from "../lib/api";
import type { Locale, Channel } from "../lib/types";

export function Settings() {
  const handleLocaleChange = async (newLocale: Locale) => {
    setLocale(newLocale);
    await api.setSetting("locale", newLocale);
  };

  return (
    <>
      <div class="settings-overlay" onClick={() => setSettingsOpen(false)} />
      <div class="settings-panel">
        <h2>{t("settings")}</h2>

        <label>{t("language")}</label>
        <select
          value={locale()}
          onChange={(e) => handleLocaleChange(e.currentTarget.value as Locale)}
        >
          {Object.entries(localeNames).map(([code, name]) => (
            <option value={code}>{name}</option>
          ))}
        </select>

        <label>{t("channels")}</label>
        <div class="channel-list">
          <For each={allChannels() ?? []}>
            {(ch: Channel) => (
              <label class="channel-toggle">
                <input
                  type="checkbox"
                  checked={ch.visible}
                  onChange={(e) => toggleChannelVisibility(ch.id, e.currentTarget.checked)}
                />
                {ch.name}
              </label>
            )}
          </For>
        </div>
      </div>
    </>
  );
}
