import { invoke, isTauri } from "@tauri-apps/api/core";
import type { Preferences } from "./preferences";

export type ReminderSnapshot = {
  preferences: Preferences | null;
  pending: string[];
  today: string;
  snoozeUntil: string | null;
  snoozed: boolean;
  nextReminder: string | null;
  confirmationSeconds: number | null;
};

export function loadReminder(): Promise<ReminderSnapshot> {
  if (!isTauri()) throw new Error("Open the BYT desktop app to use reminders.");
  return invoke<ReminderSnapshot>("load_reminder");
}
