import { createSignal, createResource } from "solid-js";
import type { Tab, Program, Favourite } from "./types";
import { api } from "./api";
import { nowTimestamp, hoursFromNow, endOfDay } from "./time";

// --- UI Signals ---
export const [activeTab, setActiveTab] = createSignal<Tab>("nu");
export const [currentTime, setCurrentTime] = createSignal(nowTimestamp());
export const [selectedWeek, setSelectedWeek] = createSignal(0);
export const [dayExpanded, setDayExpanded] = createSignal(false);
export const [popupProgram, setPopupProgram] = createSignal<Program | null>(null);
export const [settingsOpen, setSettingsOpen] = createSignal(false);

// --- Tick: update currentTime every 60s ---
setInterval(() => setCurrentTime(nowTimestamp()), 60_000);

// --- Data Resources ---
const [refreshCounter, setRefreshCounter] = createSignal(1);

export const [channels, { refetch: refetchChannels }] = createResource(
  refreshCounter,
  () => api.getChannels()
);

export const [allChannels, { refetch: refetchAllChannels }] = createResource(
  refreshCounter,
  () => api.getAllChannels()
);

export const [favourites, { refetch: refetchFavourites }] = createResource(
  refreshCounter,
  () => api.getFavourites()
);

// Programs depend on currentTime and dayExpanded
export const [programs] = createResource(
  () => ({
    from: currentTime(),
    expanded: dayExpanded(),
    tick: refreshCounter(),
  }),
  (params) => {
    const to = params.expanded
      ? endOfDay(new Date())
      : hoursFromNow(3);
    return api.getPrograms(params.from, to);
  }
);

// Favourite programs: next 5 weeks
export const [favouritePrograms, { refetch: refetchFavouritePrograms }] = createResource(
  refreshCounter,
  () => {
    const from = nowTimestamp();
    const to = from + 35 * 86400;
    return api.getFavouritePrograms(from, to);
  }
);

// --- Actions ---
export async function toggleFavourite(title: string) {
  await api.toggleFavourite(title);
  refetchFavourites();
  refetchFavouritePrograms();
}

export async function toggleChannelVisibility(channelId: string, visible: boolean) {
  await api.setChannelVisibility(channelId, visible);
  refetchChannels();
  refetchAllChannels();
}

export async function refreshData() {
  await api.refreshData();
  setRefreshCounter((c) => c + 1);
}

export function isFavourite(title: string): boolean {
  return (favourites() ?? []).some((f: Favourite) => f.title === title);
}
