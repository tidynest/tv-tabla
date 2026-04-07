export interface Channel {
  id: string;
  name: string;
  icon_url: string | null;
  visible: boolean;
  sort_order: number;
}

export interface Program {
  id: string;
  channel_id: string;
  title: string;
  description: string | null;
  category: string | null;
  start_time: number;
  end_time: number;
}

export interface Favourite {
  title: string;
  added_at: number;
}

export type Tab = "nu" | "favoriter" | "vecka";
export type Locale = "sv-SE" | "en-GB" | "pt-BR";
