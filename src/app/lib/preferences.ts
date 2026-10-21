import { invoke, isTauri } from "@tauri-apps/api/core";

export type Schedule =
  | { kind: "weekly"; days: number[] }
  | { kind: "monthly"; day: number | "last" };

export type Preferences = {
  url: string;
  schedule: Schedule;
  time: string;
  repeatInterval: number;
  startAtLogin: boolean;
  reminderSound: boolean;
};

export const defaultPreferences: Preferences = {
  url: "",
  schedule: { kind: "weekly", days: [4] },
  time: "17:00",
  repeatInterval: 5,
  startAtLogin: true,
  reminderSound: true,
};

export async function loadPreferences(): Promise<Preferences | null> {
  if (!isTauri()) throw new Error("Open the BYT desktop app to load and save settings.");
  return invoke<Preferences | null>("load_preferences");
}

export async function savePreferences(preferences: Preferences): Promise<Preferences> {
  return invoke<Preferences>("save_preferences", { preferences });
}

export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
