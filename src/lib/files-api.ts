import { invoke } from "@tauri-apps/api/core";

export type ManagedAttachment = {
  id: string;
  meliponaryId: string;
  originalName: string;
  extension?: string | null;
  mimeType?: string | null;
  byteSize: number;
  description?: string | null;
  notes?: string | null;
  createdAt: string;
  fileExists: boolean;
};

export type ManagedFileIssue = {
  kind: string;
  recordId: string;
  label: string;
  relativePath: string;
};

export type ManagedFilesDiagnostic = {
  expectedFiles: number;
  presentFiles: number;
  missingFiles: ManagedFileIssue[];
  orphanFiles: string[];
};

export function importMeliponaryAttachment(input: { meliponaryId: string; sourcePath: string; description?: string; notes?: string }) {
  return invoke<ManagedAttachment>("import_meliponary_attachment", { input });
}

export function listMeliponaryAttachments(meliponaryId: string) {
  return invoke<ManagedAttachment[]>("list_meliponary_attachments", { meliponaryId });
}

export function updateMeliponaryAttachment(input: { id: string; description?: string; notes?: string }) {
  return invoke<ManagedAttachment>("update_meliponary_attachment", { input });
}

export function removeMeliponaryAttachment(attachmentId: string) {
  return invoke<void>("remove_meliponary_attachment", { attachmentId });
}

export function openManagedAttachment(attachmentId: string) {
  return invoke<void>("open_managed_attachment", { attachmentId });
}

export function revealManagedAttachment(attachmentId: string) {
  return invoke<void>("reveal_managed_attachment", { attachmentId });
}

export function openInspectionPhoto(photoId: string) {
  return invoke<void>("open_inspection_photo", { photoId });
}

export function revealInspectionPhoto(photoId: string) {
  return invoke<void>("reveal_inspection_photo", { photoId });
}

export function diagnoseManagedFiles() {
  return invoke<ManagedFilesDiagnostic>("diagnose_managed_files");
}
