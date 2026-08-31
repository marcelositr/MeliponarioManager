import type { ScheduledTask, TaskPriority, TaskType, TaskView } from "../../lib/agenda-types";

export const views: Array<[TaskView, string]> = [
  ["pending", "Pendentes"],
  ["overdue", "Atrasadas"],
  ["today", "Hoje"],
  ["upcoming", "Próximas"],
  ["completed", "Concluídas"],
  ["cancelled", "Canceladas"],
  ["rescheduled", "Reagendadas"],
  ["skipped", "Ignoradas"],
  ["all", "Todas"],
];

export const taskTypes: Array<[TaskType, string]> = [
  ["inspection", "Inspeção"],
  ["feeding", "Alimentação"],
  ["maintenance", "Manutenção"],
  ["generic", "Genérica"],
];

export function AgendaStat({ label, value, attention = false, onClick }: { label: string; value: number; attention?: boolean; onClick: () => void }) {
  return <button type="button" className={attention ? "stat-card attention stat-button" : "stat-card stat-button"} onClick={onClick}><span>{label}</span><strong>{value}</strong></button>;
}

export function normalizeDateTime(value?: string) {
  if (!value) return undefined;
  const normalized = value.replace("T", " ");
  return normalized.length === 16 ? `${normalized}:00` : normalized;
}

export function toInputDateTime(value: string) {
  return value.replace(" ", "T").slice(0, 16);
}

export function taskTypeLabel(value: TaskType) {
  return taskTypes.find(([key]) => key === value)?.[1] || value;
}

export function priorityLabel(value: TaskPriority) {
  return value === "critical" ? "Crítica" : value === "attention" ? "Atenção" : "Normal";
}

export function statusLabel(value: ScheduledTask["status"]) {
  const labels = {
    pending: "Pendente",
    completed: "Concluída",
    cancelled: "Cancelada",
    rescheduled: "Reagendada",
    skipped: "Ignorada",
  };
  return labels[value];
}

export function viewLabel(value: TaskView) {
  return views.find(([key]) => key === value)?.[1] || "Agenda";
}
