import type { View } from "../types";

export type NavigationIntent = {
  view: View;
  taskId?: string;
  colonyId?: string;
  boxId?: string;
  meliponaryId?: string;
  action?: "create" | "open";
};

export type Navigate = (target: View | NavigationIntent) => void;

export function toNavigationIntent(target: View | NavigationIntent): NavigationIntent {
  return typeof target === "string" ? { view: target } : target;
}
