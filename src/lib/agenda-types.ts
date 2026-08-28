export type TaskType = "inspection" | "feeding" | "maintenance" | "generic";
export type TaskStatus = "pending" | "completed" | "cancelled" | "rescheduled" | "skipped";
export type TaskPriority = "normal" | "attention" | "critical";
export type TaskView = "all" | "pending" | "overdue" | "today" | "upcoming" | "completed" | "cancelled" | "rescheduled" | "skipped";

export type ScheduledTask = {
  id: string;
  meliponaryId: string;
  meliponaryName: string;
  colonyId?: string | null;
  colonyCode?: string | null;
  boxId?: string | null;
  boxCode?: string | null;
  taskType: TaskType;
  title: string;
  description?: string | null;
  scheduledFor: string;
  status: TaskStatus;
  priority: TaskPriority;
  sourceType?: string | null;
  sourceId?: string | null;
  completedAt?: string | null;
  completedByType?: string | null;
  completedById?: string | null;
  cancelledAt?: string | null;
  cancellationReason?: string | null;
  skippedAt?: string | null;
  skipReason?: string | null;
  rescheduledFromId?: string | null;
  rescheduleReason?: string | null;
  createdAt: string;
  updatedAt: string;
  overdue: boolean;
  today: boolean;
};

export type AgendaSummary = { overdue: number; today: number; nextSevenDays: number; future: number };
export type TaskQuery = {
  view?: TaskView;
  meliponaryId?: string;
  colonyId?: string;
  boxId?: string;
  taskType?: TaskType;
  priority?: TaskPriority;
  search?: string;
};
export type CreateTaskInput = {
  meliponaryId: string;
  colonyId?: string;
  boxId?: string;
  taskType: TaskType;
  title: string;
  description?: string;
  scheduledFor: string;
  priority?: TaskPriority;
};
export type RescheduleTaskInput = { id: string; scheduledFor: string; reason?: string };
export type TaskReasonInput = { id: string; reason: string };
export type DuplicateTaskInput = { id: string; scheduledFor: string };
export type TaskCompletion = { task: ScheduledTask; factType: string; factId: string };
export type CompleteInspectionTaskInput = {
  taskId: string; inspectedAt?: string; strength?: string; queenPresent?: boolean | null;
  layingStatus?: string; foodReserves?: string; broodStatus?: string; pestsNotes?: string;
  observations?: string; actionsTaken?: string; nextInspectionAt?: string;
};
export type CompleteFeedingTaskInput = {
  taskId: string; fedAt?: string; foodType: string; quantity?: number; unit?: string;
  responseNotes?: string; notes?: string; nextFeedingAt?: string;
};
export type CompleteMaintenanceTaskInput = {
  taskId: string; maintainedAt?: string; maintenanceType: string; description?: string;
  performedBy?: string; cost?: number; nextMaintenanceAt?: string;
};

export type ColonyRecordCenter = {
  id: string; code: string; speciesName: string; meliponaryId: string; meliponaryName: string;
  currentBoxCode?: string | null; status: string; originType: string; originNotes?: string | null;
  installedAt?: string | null; latestInspectionAt?: string | null; latestStrength?: string | null;
  latestFeedingAt?: string | null; pendingTasks: number; overdueTasks: number; currentAlerts: number;
  nextTaskTitle?: string | null; nextTaskAt?: string | null;
};
export type BoxRecordCenter = {
  id: string; code: string; meliponaryId: string; meliponaryName: string; status: string;
  currentColonyCode?: string | null; model?: string | null; material?: string | null;
  locationNote?: string | null; occupancyRecords: number; maintenanceRecords: number;
  pendingTasks: number; nextMaintenanceAt?: string | null;
};
export type MeliponaryRecordCenter = {
  id: string; name: string; responsibleName?: string | null; location?: string | null;
  archivedAt?: string | null; colonies: number; boxes: number; pendingTasks: number;
  overdueTasks: number; alerts: number; recentProductionRecords: number;
};
export type BoxOccupancyHistory = {
  id: string; colonyId: string; colonyCode: string; startedAt: string; endedAt?: string | null;
  reason?: string | null; notes?: string | null; correctedAt?: string | null;
};
