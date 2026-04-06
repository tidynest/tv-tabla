import { invoke } from "@tauri-apps/api/core";
import type { Channel, Program, Favourite } from "./types";

export const api = {
  getChannels: () => invoke<Channel[]>("get_channels"),
  getAllChannels: () => invoke<Channel[]>("get_all_channels"),
  getPrograms: (from: number, to: number) => invoke<Program[]>("get_programs", { from, to }),
  getFavourites: () => invoke<Favourite[]>("get_favourites"),
  getFavouritePrograms: (from: number, to: number) => invoke<Program[]>("get_favourite_programs", { from, to }),
  toggleFavourite: (title: string) => invoke<boolean>("toggle_favourite", { title }),
  setChannelVisibility: (channelId: string, visible: boolean) => invoke<void>("set_channel_visibility", { channelId, visible }),
  getSetting: (key: string) => invoke<string | null>("get_setting", { key }),
  setSetting: (key: string, value: string) => invoke<void>("set_setting", { key, value }),
  refreshData: () => invoke<void>("refresh_data"),
  getCacheAge: () => invoke<number | null>("get_cache_age"),
};
