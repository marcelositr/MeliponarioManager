import type { TaskPriority, TaskType } from "../../lib/agenda-types";

export type CreateForm = {
  meliponaryId: string;
  colonyId: string;
  boxId: string;
  taskType: TaskType;
  title: string;
  description: string;
  scheduledFor: string;
  priority: TaskPriority;
};

export type ExecuteForm = {
  occurredAt: string;
  nextAt: string;
  strength: string;
  foodType: string;
  quantity: string;
  unit: string;
  maintenanceType: string;
  description: string;
};

export const emptyExecute: ExecuteForm = {
  occurredAt: "",
  nextAt: "",
  strength: "unknown",
  foodType: "",
  quantity: "",
  unit: "",
  maintenanceType: "inspection",
  description: "",
};
