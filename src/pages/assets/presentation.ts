export const maintenanceTypes = [
  ["cleaning", "Limpeza"],
  ["repair", "Reparo"],
  ["painting", "Pintura"],
  ["waterproofing", "Impermeabilização"],
  ["roof", "Cobertura"],
  ["entrance", "Entrada"],
  ["internal_structure", "Estrutura interna"],
  ["inspection", "Revisão da caixa"],
  ["other", "Outro"],
] as const;

export function normalizeDateTime(value?: string) {
  if (!value) return undefined;
  const normalized = value.replace("T", " ");
  return normalized.length === 16 ? `${normalized}:00` : normalized;
}

export function toInputDateTime(value: string) {
  return value.replace(" ", "T").slice(0, 16);
}

export function formatBytes(value: number) {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / (1024 * 1024)).toFixed(1)} MB`;
}

export function fileNameFromPath(value: string) {
  return value.split(/[\\/]/).filter(Boolean).pop() || "Arquivo selecionado";
}

export function maintenanceLabel(value: string) {
  return maintenanceTypes.find(([key]) => key === value)?.[1] || value;
}
