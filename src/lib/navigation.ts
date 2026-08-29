import type { View } from "../types";

export type NavigationIntent = {
  view: View;
  taskId?: string | null;
  colonyId?: string | null;
  boxId?: string | null;
  meliponaryId?: string | null;
  action?: "create" | "open";
};

export type Navigate = (target: View | NavigationIntent) => void;

export function toNavigationIntent(target: View | NavigationIntent): NavigationIntent {
  return typeof target === "string" ? { view: target } : target;
}

export function reconcileManualMeliponaryChange(intent: NavigationIntent): NavigationIntent {
  return { view: intent.view };
}
