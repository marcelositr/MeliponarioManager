import { invoke } from "@tauri-apps/api/core";
import type { InspectionPhoto } from "../types";
import type {
  AgendaSummary,
  BoxOccupancyHistory,
  BoxRecordCenter,
  ColonyRecordCenter,
  CompleteFeedingTaskInput,
  CompleteInspectionTaskInput,
  CompleteMaintenanceTaskInput,
  CreateTaskInput,
  DuplicateTaskInput,
  MeliponaryRecordCenter,
  RescheduleTaskInput,
  ScheduledTask,
  TaskCompletion,
  TaskQuery,
  TaskReasonInput,
} from "./agenda-types";

export const listTasks = (query: TaskQuery) => invoke<ScheduledTask[]>("list_tasks", { query });
export const getTask = (taskId: string) => invoke<ScheduledTask>("get_task", { taskId });
export const getAgendaSummary = (meliponaryId?: string) => invoke<AgendaSummary>("get_agenda_summary", { meliponaryId });
export const createTask = (input: CreateTaskInput) => invoke<ScheduledTask>("create_task", { input });
export const rescheduleTask = (input: RescheduleTaskInput) => invoke<ScheduledTask>("reschedule_task", { input });
export const cancelTask = (input: TaskReasonInput) => invoke<ScheduledTask>("cancel_task", { input });
export const skipTask = (input: TaskReasonInput) => invoke<ScheduledTask>("skip_task", { input });
export const completeGenericTask = (taskId: string) => invoke<ScheduledTask>("complete_generic_task", { taskId });
export const duplicateTask = (input: DuplicateTaskInput) => invoke<ScheduledTask>("duplicate_task", { input });
export const completeInspectionTask = (input: CompleteInspectionTaskInput) => invoke<TaskCompletion>("complete_inspection_task", { input });
export const completeFeedingTask = (input: CompleteFeedingTaskInput) => invoke<TaskCompletion>("complete_feeding_task", { input });
export const completeMaintenanceTask = (input: CompleteMaintenanceTaskInput) => invoke<TaskCompletion>("complete_maintenance_task", { input });

export const getColonyRecordCenter = (colonyId: string) => invoke<ColonyRecordCenter>("get_colony_record_center", { colonyId });
export const getBoxRecordCenter = (boxId: string) => invoke<BoxRecordCenter>("get_box_record_center", { boxId });
export const getMeliponaryRecordCenter = (meliponaryId: string) => invoke<MeliponaryRecordCenter>("get_meliponary_record_center", { meliponaryId });
export const listBoxOccupancies = (boxId: string) => invoke<BoxOccupancyHistory[]>("list_box_occupancies", { boxId });
export const listBoxContextPhotos = (boxId: string) => invoke<InspectionPhoto[]>("list_box_context_photos", { boxId });
