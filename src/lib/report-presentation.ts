export function defaultReportPeriod(now = new Date()) {
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return { startDate: `${year}-${month}-01`, endDate: `${year}-${month}-${day}` };
}

export function sanitizeReportFilename(value: string) {
  return value
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[<>:"/\\|?*\u0000-\u001F]/g, "-")
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "")
    .toLowerCase();
}

export function reportFilename(kind: string, startDate: string, endDate: string, context?: string) {
  const base = [kind, context, `${startDate}-a-${endDate}`].filter(Boolean).join("-");
  return `${sanitizeReportFilename(base) || "relatorio"}.csv`;
}

export function formatReportNumber(value: number) {
  return new Intl.NumberFormat("pt-BR", { maximumFractionDigits: 3 }).format(value);
}

export function formatBrl(value: number) {
  return new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" }).format(value);
}
