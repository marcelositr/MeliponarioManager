export type ThemeMode = "light" | "dark" | "system";
export type ResolvedTheme = "light" | "dark";

export const UI_STORAGE = {
  theme: "meliponario.ui.theme",
  sidebarCollapsed: "meliponario.ui.sidebarCollapsed",
  activeMeliponary: "meliponario.ui.activeMeliponary",
} as const;

export function normalizeTheme(value: string | null): ThemeMode {
  return value === "light" || value === "dark" || value === "system" ? value : "system";
}

export function resolveTheme(mode: ThemeMode, systemPrefersDark: boolean): ResolvedTheme {
  if (mode === "system") return systemPrefersDark ? "dark" : "light";
  return mode;
}

export function normalizeActiveMeliponary(value: string | null, availableIds: readonly string[]): string {
  if (!value || value === "all") return "all";
  return availableIds.includes(value) ? value : "all";
}

export function readSidebarCollapsed(value: string | null): boolean {
  return value === "1";
}
