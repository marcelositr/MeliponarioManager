const technicalMarkers = [
  "sqlx",
  "sqlite",
  "database error",
  "constraint failed",
  "rust",
  "invoke",
  "tauri",
  "stack",
  "panic",
];

export function formatDateTimeBr(value?: string | null) {
  if (!value) return "—";
  const normalized = value.replace("T", " ").slice(0, 19);
  const match = normalized.match(/^(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2})(?::\d{2})?$/);
  if (!match) return value;
  const [, year, month, day, hour, minute] = match;
  return `${day}/${month}/${year} ${hour}:${minute}`;
}

export function publicError(error: unknown, fallback = "Não foi possível concluir a operação.") {
  const raw = error instanceof Error ? error.message : typeof error === "string" ? error : "";
  const text = raw.replace(/^Error:\s*/i, "").trim();
  if (!text) return fallback;
  const lower = text.toLowerCase();
  if (technicalMarkers.some((marker) => lower.includes(marker))) return fallback;
  if (text.length > 220) return fallback;
  return text;
}

export function linkedFactLabel(type?: string | null) {
  if (!type) return "Registro operacional vinculado";
  const labels: Record<string, string> = {
    inspection: "Inspeção vinculada",
    feeding: "Alimentação vinculada",
    maintenance: "Manutenção vinculada",
    production: "Produção vinculada",
    event: "Evento vinculado",
  };
  return labels[type] ?? "Registro operacional vinculado";
}
