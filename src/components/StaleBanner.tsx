import { createResource, Show } from "solid-js";
import { t } from "../i18n";
import { api } from "../lib/api";

const STALE_THRESHOLD = 6 * 3600;

export function StaleBanner() {
  const [cacheAge] = createResource(
    () => true,
    () => api.getCacheAge(),
    { initialValue: null }
  );

  const isStale = () => {
    const age = cacheAge();
    return age !== null && age !== undefined && age > STALE_THRESHOLD;
  };

  const hoursAgo = () => {
    const age = cacheAge();
    if (!age) return "";
    return `${Math.floor(age / 3600)}h`;
  };

  return (
    <Show when={isStale()}>
      <div class="stale-banner">
        {t("stale_data")} {hoursAgo()}
      </div>
    </Show>
  );
}
