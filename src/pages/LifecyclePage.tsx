import { useEffect, useMemo, useState, type FormEvent } from "react";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { ReasonDialog } from "../components/ReasonDialog";
import { RecordActions } from "../components/RecordActions";
import { RecordStateBadge } from "../components/RecordStateBadge";
import type { RecordStateMap } from "../hooks/useAppData";
import { listColonyLifecycle } from "../lib/api";
import type { ChangeColonyLifecycleInput, Colony, ColonyLifecycleRecord, ReverseRecordInput } from "../types";

type Props = {
  colonies: Colony[];
  busy: boolean;
  recordStateMap: RecordStateMap;
  onChange: (input: ChangeColonyLifecycleInput) => Promise<boolean>;
  onReverse: (input: ReverseRecordInput) => Promise<boolean>;
};
const initialForm: ChangeColonyLifecycleInput = { colonyId: "", action: "deactivate", occurredAt: "", reason: "", notes: "" };

export function LifecyclePage({ colonies, busy, recordStateMap, onChange, onReverse }: Props) {
  const [selectedColonyId, setSelectedColonyId] = useState("");
  const [form, setForm] = useState<ChangeColonyLifecycleInput>(initialForm);
  const [items, setItems] = useState<ColonyLifecycleRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [open, setOpen] = useState(false);
  const [lossConfirm, setLossConfirm] = useState(false);
  const [detail, setDetail] = useState<ColonyLifecycleRecord | null>(null);
  const [reverseTarget, setReverseTarget] = useState<ColonyLifecycleRecord | null>(null);
  const selectedColony = colonies.find((colony) => colony.id === form.colonyId);
  const selectedForHistory = colonies.find((colony) => colony.id === selectedColonyId);
  const allowedActions = useMemo(() => actionsForStatus(selectedColony?.status), [selectedColony?.status]);

  useEffect(() => { if (allowedActions.length > 0 && !allowedActions.some(([value]) => value === form.action)) setForm((current) => ({ ...current, action: allowedActions[0][0] })); }, [allowedActions, form.action]);
  useEffect(() => { void reload(selectedColonyId); }, [selectedColonyId]);
  async function reload(colonyId = selectedColonyId) { if (!colonyId) { setItems([]); return; } setLoading(true); try { setItems(await listColonyLifecycle(colonyId)); } finally { setLoading(false); } }
  function openChange() { const colonyId = selectedColonyId; const colony = colonies.find((item) => item.id === colonyId); const actions = actionsForStatus(colony?.status); setForm({ ...initialForm, colonyId, action: actions[0]?.[0] || "deactivate" }); setOpen(true); }
  async function commitChange() { const input = { ...form, occurredAt: normalizeDateTime(form.occurredAt) }; if (await onChange(input)) { setSelectedColonyId(form.colonyId); setLossConfirm(false); setOpen(false); await reload(form.colonyId); } }
  async function submit(event: FormEvent) { event.preventDefault(); if (!allowedActions.some(([value]) => value === form.action)) return; if (form.action === "loss") { setLossConfirm(true); return; } await commitChange(); }

  return <div className="page-stack">
    <PageToolbar title="Ciclo de vida" description="Baixa, inativação e reativação preservando todo o histórico." count={selectedColonyId ? `${items.length} transições` : `${colonies.length} colônias`} primaryAction={{ label: "Alterar situação", onClick: openChange, disabled: busy || !selectedColonyId || actionsForStatus(selectedForHistory?.status).length === 0 }}><label className="toolbar-select"><span className="sr-only">Colônia</span><select value={selectedColonyId} onChange={(e) => setSelectedColonyId(e.target.value)}><option value="">Selecione uma colônia...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} · {statusLabel(colony.status)}</option>)}</select></label></PageToolbar>
    <section className="panel wide-list"><div className="panel-heading"><h2>Histórico de transições</h2><p>Reversão é controlada pelo backend e só ocorre quando não existem consequências posteriores incompatíveis.</p></div>{!selectedColonyId ? <div className="empty-list">Selecione uma colônia na toolbar.</div> : loading ? <div className="empty-list">Carregando histórico...</div> : items.length === 0 ? <div className="empty-list">Nenhuma transição registrada. A entrada inicial continua na timeline.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Data</th><th>Ação</th><th>Transição</th><th>Estado</th><th>Ações</th></tr></thead><tbody>{items.map((item) => { const state = recordStateMap.get(`lifecycle:${item.id}`); return <tr key={item.id} className={state?.reversedAt ? "voided-row" : undefined}><td><strong>{formatDateTime(item.occurredAt)}</strong></td><td>{actionLabel(item.action)}</td><td>{statusLabel(item.previousStatus)} → {statusLabel(item.newStatus)}</td><td><RecordStateBadge state={state} /></td><td><RecordActions busy={busy} onOpen={() => setDetail(item)} secondary={[{ label: "Reverter", onClick: () => setReverseTarget(item), disabled: Boolean(state?.reversedAt), danger: true }]} /></td></tr>; })}</tbody></table></div>}</section>

    <Dialog open={open} onClose={() => !busy && setOpen(false)} title="Alterar situação da colônia" description="Perda e inativação encerram a ocupação atual. Reativação não atribui uma caixa automaticamente." size="medium"><form className="form-grid" onSubmit={submit}><label className="field full"><span>Colônia</span><select autoFocus required value={form.colonyId} onChange={(e) => setForm({ ...form, colonyId: e.target.value })}><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} · {statusLabel(colony.status)}</option>)}</select></label>{selectedColony && allowedActions.length === 0 ? <div className="inline-notice field full">Esta situação não permite nova transição pela rotina de ciclo de vida.</div> : <><label className="field"><span>Ação</span><select value={form.action} onChange={(e) => setForm({ ...form, action: e.target.value })}>{allowedActions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label><label className="field"><span>Data e hora</span><input type="datetime-local" value={form.occurredAt} onChange={(e) => setForm({ ...form, occurredAt: e.target.value })} /></label><label className="field full"><span>Motivo</span><input value={form.reason} onChange={(e) => setForm({ ...form, reason: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={3} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} /></label>{form.action === "loss" && <div className="inline-notice field full">Baixa por perda é terminal até que uma reversão segura seja explicitamente executada.</div>}{form.action === "reactivate" && <div className="inline-notice field full">Reativar devolve a situação para ativa, mas a caixa deve ser atribuída separadamente.</div>}<div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setOpen(false)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !form.colonyId || allowedActions.length === 0}>{busy ? "Salvando..." : actionButton(form.action)}</button></div></>}</form></Dialog>
    <Dialog open={Boolean(detail)} onClose={() => setDetail(null)} title="Transição do ciclo de vida" description={detail ? `${detail.colonyCode} · ${formatDateTime(detail.occurredAt)}` : ""} size="medium">{detail && <div className="detail-grid"><div><span>Ação</span><strong>{actionLabel(detail.action)}</strong></div><div><span>Caixa</span><strong>{detail.boxCode || "—"}</strong></div><div><span>Anterior</span><strong>{statusLabel(detail.previousStatus)}</strong></div><div><span>Nova</span><strong>{statusLabel(detail.newStatus)}</strong></div><div className="full"><span>Motivo</span><p>{detail.reason || "—"}</p></div><div className="full"><span>Observações</span><p>{detail.notes || "—"}</p></div></div>}</Dialog>
    <ConfirmDialog open={lossConfirm} title="Registrar baixa da colônia?" consequence="Esta operação encerra a ocupação atual da caixa e registra a perda. Uma eventual reversão posterior só será aceita se o estado histórico continuar seguro." confirmLabel="Registrar baixa" danger busy={busy} onCancel={() => setLossConfirm(false)} onConfirm={() => { void commitChange(); }} />
    <ReasonDialog open={Boolean(reverseTarget)} title="Reverter transição?" description={reverseTarget ? `${reverseTarget.colonyCode} · ${actionLabel(reverseTarget.action)} · ${formatDateTime(reverseTarget.occurredAt)}` : ""} confirmLabel="Reverter transição" consequence="A operação tenta restaurar o estado anterior e, quando aplicável, a ocupação anterior. Se houver qualquer consequência posterior incompatível, o backend bloqueará a reversão sem alterar dados." danger busy={busy} onClose={() => setReverseTarget(null)} onConfirm={async (reason) => { if (!reverseTarget) return false; const ok = await onReverse({ id: reverseTarget.id, reason }); if (ok) await reload(); return ok; }} />
  </div>;
}
function actionsForStatus(status?: string): Array<[string, string]> { if (status === "inactive") return [["reactivate", "Reativar colônia"]]; if (status === "active" || status === "weak" || status === "recovering") return [["deactivate", "Inativar colônia"], ["loss", "Baixa por perda"]]; return []; }
function normalizeDateTime(value?: string) { if (!value) return undefined; const normalized = value.replace("T", " "); return normalized.length === 16 ? `${normalized}:00` : normalized; }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function actionLabel(value: string) { return value === "loss" ? "Baixa por perda" : value === "deactivate" ? "Colônia inativada" : "Colônia reativada"; }
function actionButton(value?: string) { return value === "loss" ? "Registrar baixa" : value === "reactivate" ? "Reativar colônia" : "Inativar colônia"; }
function statusLabel(value?: string) { const labels: Record<string, string> = { active: "Ativa", weak: "Fraca", recovering: "Em recuperação", inactive: "Inativa", lost: "Perdida", transferred: "Transferida" }; return value ? labels[value] || value : ""; }
