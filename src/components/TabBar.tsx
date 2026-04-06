import { t } from "../i18n";
import { activeTab, setActiveTab, settingsOpen, setSettingsOpen } from "../lib/state";
import type { Tab } from "../lib/types";

export function TabBar() {
  const tabs: { id: Tab; label: () => string }[] = [
    { id: "nu", label: () => t("tab_now") },
    { id: "favoriter", label: () => t("tab_favourites") },
    { id: "vecka", label: () => t("tab_week") },
  ];

  return (
    <nav class="tab-bar">
      {tabs.map((tab) => (
        <button
          class={activeTab() === tab.id ? "active" : ""}
          onClick={() => setActiveTab(tab.id)}
        >
          {tab.label()}
        </button>
      ))}
      <div class="spacer" />
      <button
        class="settings-btn"
        onClick={() => setSettingsOpen(!settingsOpen())}
        title={t("settings")}
      >
        &#9881;
      </button>
    </nav>
  );
}
