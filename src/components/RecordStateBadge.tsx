import type { RecordAdminState } from "../types";

export function RecordStateBadge({ state }: { state?: RecordAdminState }) {
  if (!state) return null;
  if (state.reversedAt) return <span className="badge status-reversed" title={state.reversalReason || undefined}>Revertido</span>;
  if (state.voidedAt) return <span className="badge status-voided" title={state.voidReason || undefined}>Anulado</span>;
  if (state.correctedAt) return <span className="badge status-corrected">Corrigido</span>;
  return null;
}
