import { useEffect, useMemo, useState, type FormEvent } from "react";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { listColonyLifecycle } from "../lib/api";
import type { ChangeColonyLifecycleInput, Colony, ColonyLifecycleRecord } from "../types";

type LifecyclePageProps = { colonies: Colony[]; busy: boolean; onChange: (input: ChangeColonyLifecycleInput) => Promise<boolean>; };
const initialForm: ChangeColonyLifecycleInput = { colonyId: "", action: "deactivate", occurredAt: "", reason: "", notes: "" };

export function LifecyclePage({ colonies, busy, onChange }: LifecyclePageProps) {
  const [selectedColonyId, setSelectedColonyId] = useState("");
  const [form, setForm] = useState<ChangeColonyLifecycleInput>(initialForm);
  const [items, setItems] = useState<ColonyLifecycleRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [open, setOpen] = useState(false);
  const [lossConfirm, setLossConfirm] = useState(false);
  const selectedColony = colonies.find((colony) => colony.id === form.colonyId);
  const selectedForHistory = colonies.find((colony) => colony.id === selectedColonyId);
  const allowedActions = useMemo(() => actionsForStatus(selectedColony?.status), [selectedColony?.status]);

  useEffect(() => { if (allowedActions.length > 0 && !allowedActions.some(([value]) => value === form.action)) setForm((current) => ({ ...current, action: allowedActions[0][0] })); }, [allowedActions, form.action]);
  useEffect(() => { let cancelled = false; if (!selectedColonyId) { setItems([]); return; } setLoading(true); listColonyLifecycle(selectedColonyId).then((records) => { if (!cancelled) setItems(records); }).finally(() => { if (!cancelled) setLoading(false); }); return () => { cancelled = true; }; }, [selectedColonyId]);
  async function reload(colonyId = selectedColonyId) { if (colonyId) setItems(await listColonyLifecycle(colonyId)); }
  function openChange() { const colonyId = selectedColonyId; const colony = colonies.find((item) => item.id === colonyId); const actions = actionsForStatus(colony?.status); setForm({ ...initialForm, colonyId, action: actions[0]?.[0] || "deactivate" }); setOpen(true); }
  async function commitChange() { const input = { ...form, occurredAt: normalizeDateTime(form.occurredAt) }; if (await onChange(input)) { setSelectedColonyId(form.colonyId); setLossConfirm(false); setOpen(false); await reload(form.colonyId); } }
  async function submit(event: FormEvent) { event.preventDefault(); if (!allowedActions.some(([value]) => value === form.action)) return; if (form.action === "loss") { setLossConfirm(true); return; } await commitChange(); }

  return <div className="page-stack">
    <PageToolbar title="Ciclo de vida" description="Baixa, inativação e reativação preservando todo o histórico." count={selectedColonyId ? `${items.length} transições` : `${colonies.length} colônias`} primaryAction={{ label: "Alterar situação", onClick: openChange, disabled: busy || !selectedColonyId || actionsForStatus(selectedForHistory?.status).length === 0 }}><label className="toolbar-select"><span className="sr-only">Colônia</span><select value={selectedColonyId} onChange={(e) => setSelectedColonyId(e.target.value)}><option value="">Selecione uma colônia...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} · {statusLabel(colony.status)}</option>)}</select></label></PageToolbar>
    <section className="panel wide-list"><div className="panel-heading"><h2>Histórico de transições</h2><p>Preserva situação anterior, nova situação, caixa da data e motivo informado.</p></div>{!selectedColonyId ? <div className="empty-list">Selecione uma colônia na toolbar.</div> : loading ? <div className="empty-list">Carregando histórico...</div> : items.length === 0 ? <div className="empty-list">Nenhuma transição registrada. A entrada inicial continua na timeline.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Data</th><th>Ação</th><th>Caixa</th><th>Transição</th><th>Motivo</th></tr></thead><tbody>{items.map((item) => <tr key={item.id}><td><strong>{formatDateTime(item.occurredAt)}</strong></td><td>{actionLabel(item.action)}</td><td>{item.boxCode || "—"}</td><td>{statusLabel(item.previousStatus)} → {statusLabel(item.newStatus)}</td><td>{item.reason || "—"}</td></tr>)}</tbody></table></div>}</section>
    <Dialog open={open} onClose={() => !busy && setOpen(false)} title="Alterar situação da colônia" description="Perda e inativação encerram a ocupação atual. Reativação não atribui uma caixa automaticamente." size="medium"><form className="form-grid" onSubmit={submit}><label className="field full"><span>Colônia</span><select autoFocus required value={form.colonyId} onChange={(e) => setForm({ ...form, colonyId: e.target.value })}><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} · {statusLabel(colony.status)}</option>)}</select></label>{selectedColony && allowedActions.length === 0 ? <div className="inline-notice field full">Esta situação não permite nova transição pela rotina de ciclo de vida.</div> : <><label className="field"><span>Ação</span><select value={form.action} onChange={(e) => setForm({ ...form, action: e.target.value })}>{allowedActions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label><label className="field"><span>Data e hora</span><input type="datetime-local" value={form.occurredAt} onChange={(e) => setForm({ ...form, occurredAt: e.target.value })} /></label><label className="field full"><span>Motivo</span><input value={form.reason} onChange={(e) => setForm({ ...form, reason: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={3} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} /></label>{form.action === "loss" && <div className="inline-notice field full">Baixa por perda é terminal. A ocupação ativa da caixa será encerrada e a colônia não poderá ser reativada por esta rotina.</div>}{form.action === "reactivate" && <div className="inline-notice field full">Reativar devolve a situação para ativa, mas a caixa deve ser atribuída separadamente.</div>}<div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setOpen(false)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !form.colonyId || allowedActions.length === 0}>{busy ? "Salvando..." : actionButton(form.action)}</button></div></>}</form></Dialog>
    <ConfirmDialog open={lossConfirm} title="Registrar baixa da colônia?" consequence="Esta operação encerra a ocupação atual da caixa, registra a perda como fato terminal e preserva toda a informação anterior no histórico." confirmLabel="Registrar baixa" danger busy={busy} onCancel={() => setLossConfirm(false)} onConfirm={() => { void commitChange(); }} />
  </div>;
}
function actionsForStatus(status?: string): Array<[string, string]> { if (status === "inactive") return [["reactivate", "Reativar colônia"]]; if (status === "active" || status === "weak" || status === "recovering") return [["deactivate", "Inativar colônia"], ["loss", "Baixa por perda"]]; return []; }
function normalizeDateTime(value?: string) { if (!value) return undefined; const normalized = value.replace("T", " "); return normalized.length === 16 ? `${normalized}:00` : normalized; }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function actionLabel(value: string) { return value === "loss" ? "Baixa por perda" : value === "deactivate" ? "Colônia inativada" : "Colônia reativada"; }
function actionButton(value?: string) { return value === "loss" ? "Registrar baixa" : value === "reactivate" ? "Reativar colônia" : "Inativar colônia"; }
function statusLabel(value?: string) { const labels: Record<string, string> = { active: "Ativa", weak: "Fraca", recovering: "Em recuperação", inactive: "Inativa", lost: "Perdida", transferred: "Transferida" }; return value ? labels[value] || value : ""; }
