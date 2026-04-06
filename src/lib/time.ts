import type { Locale } from "./types";

export function nowTimestamp(): number {
  return Math.floor(Date.now() / 1000);
}

export function hoursFromNow(hours: number): number {
  return nowTimestamp() + hours * 3600;
}

export function startOfDay(date: Date): number {
  const d = new Date(date);
  d.setHours(0, 0, 0, 0);
  return Math.floor(d.getTime() / 1000);
}

export function endOfDay(date: Date): number {
  const d = new Date(date);
  d.setHours(23, 59, 59, 999);
  return Math.floor(d.getTime() / 1000);
}

export function formatTime(ts: number, locale: Locale): string {
  const date = new Date(ts * 1000);
  return date.toLocaleTimeString(locale, { hour: "2-digit", minute: "2-digit" });
}

export function formatDate(ts: number, locale: Locale): string {
  const date = new Date(ts * 1000);
  return date.toLocaleDateString(locale, { weekday: "short", month: "short", day: "numeric" });
}

export function relativeDay(ts: number, locale: Locale): string {
  const now = new Date();
  const target = new Date(ts * 1000);
  const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  const targetStart = new Date(target.getFullYear(), target.getMonth(), target.getDate()).getTime();
  const diffDays = Math.floor((targetStart - todayStart) / 86400000);

  const labels: Record<Locale, { today: string; tomorrow: string }> = {
    "sv-SE": { today: "Idag", tomorrow: "Imorgon" },
    "en-GB": { today: "Today", tomorrow: "Tomorrow" },
    "pt-BR": { today: "Hoje", tomorrow: "Amanha" },
  };

  const l = labels[locale];
  if (diffDays === 0) return l.today;
  if (diffDays === 1) return l.tomorrow;
  return new Date(ts * 1000).toLocaleDateString(locale, { weekday: "long" });
}

export function weekNumber(date: Date): number {
  const d = new Date(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()));
  const dayNum = d.getUTCDay() || 7;
  d.setUTCDate(d.getUTCDate() + 4 - dayNum);
  const yearStart = new Date(Date.UTC(d.getUTCFullYear(), 0, 1));
  return Math.ceil(((d.getTime() - yearStart.getTime()) / 86400000 + 1) / 7);
}

export function weekStartDate(weeksFromNow: number): Date {
  const now = new Date();
  const dayOfWeek = now.getDay() || 7;
  const monday = new Date(now);
  monday.setDate(now.getDate() - dayOfWeek + 1 + weeksFromNow * 7);
  monday.setHours(0, 0, 0, 0);
  return monday;
}
