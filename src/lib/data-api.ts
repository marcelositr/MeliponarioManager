import { invoke } from "@tauri-apps/api/core";
import type { GeneratedArtifact, RestoreStageResult } from "../data-types";

export const createFullBackup = () => invoke<GeneratedArtifact>("create_full_backup");
export const exportPortableJson = () => invoke<GeneratedArtifact>("export_portable_json");
export const generateManagementReport = () => invoke<GeneratedArtifact>("generate_management_report");
export const stageRestore = (backupPath: string) => invoke<RestoreStageResult>("stage_restore", { backupPath });
