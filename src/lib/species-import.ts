import { invoke } from "@tauri-apps/api/core";

export type SpeciesImportPreviewRow = {
  rowNumber: number;
  commonName: string;
  scientificName?: string | null;
  genus?: string | null;
  status: "new" | "duplicate" | "invalid";
  message?: string | null;
};

export type SpeciesImportPreview = {
  fileName: string;
  totalRows: number;
  newRows: number;
  duplicateRows: number;
  invalidRows: number;
  rows: SpeciesImportPreviewRow[];
  truncated: boolean;
};

export type SpeciesImportResult = {
  totalRows: number;
  importedRows: number;
  duplicateRows: number;
};

export const analyzeSpeciesCsv = (sourcePath: string) =>
  invoke<SpeciesImportPreview>("analyze_species_csv", { sourcePath });

export const importSpeciesCsv = (sourcePath: string) =>
  invoke<SpeciesImportResult>("import_species_csv", { sourcePath });
