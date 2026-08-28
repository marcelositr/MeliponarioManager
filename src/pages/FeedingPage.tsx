import { useEffect, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { ReasonDialog } from "../components/ReasonDialog";
import { RecordActions } from "../components/RecordActions";
import { RecordStateBadge } from "../components/RecordStateBadge";
import type { RecordStateMap } from "../hooks/useAppData";
import { listColonyFeedings } from "../lib/api";
import type { Colony, CorrectFeedingInput, CreateFeedingInput, Feeding, VoidRecordInput } from "../types";

type Props = {
  colonies: Colony[];
  busy: boolean;
  recordStateMap: RecordStateMap;
  onCreate: (input: CreateFeedingInput) => Promise<boolean>;
  onCorrect: (input: CorrectFeedingInput) => Promise<boolean>;
  onVoid: (input: VoidRecordInput) => Promise<boolean>;
};
type FeedingForm = { colonyId: string; fedAt: string; foodType: string; quantity: string; unit: string; responseNotes: string; notes: string; nextFeedingAt: string };
const initialForm: FeedingForm = { colonyId: "", fedAt: "", foodType: "", quantity: "", unit: "", responseNotes: "", notes: "", nextFeedingAt: "" };

export function FeedingPage({ colonies, busy, recordStateMap, onCreate, onCorrect, onVoid }: Props) {
  const [selectedColonyId, setSelectedColonyId] = useState("");
  const [form, setForm] = useState<FeedingForm>(initialForm);
  const [items, setItems] = useState<Feeding[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadError, setLoadError] = useState("");
  const [open, setOpen] = useState(false);
  const [detail, setDetail] = useState<Feeding | null>(null);
  const [editForm, setEditForm] = useState<CorrectFeedingInput | null>(null);
  const [voidTarget, setVoidTarget] = useState<Feeding | null>(null);

  useEffect(() => { void reload(selectedColonyId); }, [selectedColonyId]);
  async function reload(colonyId = selectedColonyId) {
    if (!colonyId) { setItems([]); setLoadError(""); return; }
    setLoading(true); setLoadError("");
    try { setItems(await listColonyFeedings(colonyId)); }
    catch { setLoadError("Não foi possível carregar as alimentações desta colônia."); }
    finally { setLoading(false); }
  }
  function openNew() { setForm({ ...initialForm, colonyId: selectedColonyId }); setOpen(true); }
  async function submit(event: FormEvent) {
    event.preventDefault();
    const quantity = form.quantity.trim() ? Number(form.quantity) : undefined;
    const input: CreateFeedingInput = { colonyId: form.colonyId, fedAt: normalizeDateTime(form.fedAt), foodType: form.foodType, quantity, unit: form.unit.trim() || undefined, responseNotes: form.responseNotes, notes: form.notes, nextFeedingAt: normalizeDateTime(form.nextFeedingAt) };
    if (await onCreate(input)) { setSelectedColonyId(form.colonyId); setOpen(false); setForm(initialForm); await reload(form.colonyId); }
  }
  function beginCorrect(item: Feeding) {
    setEditForm({ id: item.id, reason: "", fedAt: toInputDateTime(item.fedAt), foodType: item.foodType, quantity: item.quantity ?? undefined, unit: item.unit || undefined, responseNotes: item.responseNotes || "", notes: item.notes || "", nextFeedingAt: item.nextFeedingAt ? toInputDateTime(item.nextFeedingAt) : "" });
  }
  async function submitCorrection(event: FormEvent) {
    event.preventDefault(); if (!editForm) return;
    const payload: CorrectFeedingInput = { ...editForm, fedAt: normalizeDateTime(editForm.fedAt) || editForm.fedAt, nextFeedingAt: normalizeDateTime(editForm.nextFeedingAt) };
    if (await onCorrect(payload)) { setEditForm(null); await reload(); }
  }

  return <div className="page-stack">
    <PageToolbar title="Alimentação" description="Suplementações e resposta observada por colônia." count={selectedColonyId ? `${items.length} registros` : `${colonies.length} colônias`} primaryAction={{ label: "Nova alimentação", onClick: openNew, disabled: busy || colonies.length === 0 }}><label className="toolbar-select"><span className="sr-only">Colônia</span><select value={selectedColonyId} onChange={(e) => setSelectedColonyId(e.target.value)}><option value="">Selecione uma colônia...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>)}</select></label></PageToolbar>
    {colonies.length === 0 ? <div className="inline-notice">Cadastre uma colônia antes de registrar alimentação.</div> : <section className="panel wide-list"><div className="panel-heading"><h2>Histórico da colônia</h2><p>Correções preservam a versão anterior na auditoria; anulações continuam visíveis e deixam de gerar pendências.</p></div>{!selectedColonyId ? <div className="empty-list">Selecione uma colônia na toolbar para consultar o histórico.</div> : loading ? <div className="empty-list">Carregando alimentações...</div> : loadError ? <div className="inline-notice">{loadError}</div> : items.length === 0 ? <div className="empty-list">Nenhuma alimentação registrada para esta colônia.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Data</th><th>Alimento</th><th>Quantidade</th><th>Caixa</th><th>Próxima</th><th>Estado</th><th>Ações</th></tr></thead><tbody>{items.map((item) => { const state = recordStateMap.get(`feeding:${item.id}`); return <tr key={item.id} className={state?.voidedAt ? "voided-row" : undefined}><td><strong>{formatDateTime(item.fedAt)}</strong></td><td>{item.foodType}</td><td>{quantityLabel(item)}</td><td>{item.boxCode || "Sem caixa"}</td><td>{item.nextFeedingAt ? formatDateTime(item.nextFeedingAt) : "Sem agendamento"}</td><td><RecordStateBadge state={state} /></td><td><RecordActions busy={busy} onOpen={() => setDetail(item)} onEdit={state?.voidedAt ? undefined : () => beginCorrect(item)} secondary={[{ label: "Anular", onClick: () => setVoidTarget(item), disabled: Boolean(state?.voidedAt), danger: true }]} /></td></tr>; })}</tbody></table></div>}</section>}

    <Dialog open={open} onClose={() => !busy && setOpen(false)} title="Nova alimentação" description="Quantidade e unidade são opcionais, mas devem ser coerentes quando informadas." size="large"><FeedingCreateForm form={form} setForm={setForm} colonies={colonies} busy={busy} onSubmit={submit} /></Dialog>
    <Dialog open={Boolean(detail)} onClose={() => setDetail(null)} title="Alimentação" description={detail ? `${detail.colonyCode} · ${formatDateTime(detail.fedAt)}` : ""} size="medium">{detail && <div className="detail-grid"><div><span>Caixa</span><strong>{detail.boxCode || "Sem caixa"}</strong></div><div><span>Alimento</span><strong>{detail.foodType}</strong></div><div><span>Quantidade</span><strong>{quantityLabel(detail)}</strong></div><div><span>Próxima</span><strong>{detail.nextFeedingAt ? formatDateTime(detail.nextFeedingAt) : "Sem agendamento"}</strong></div><div className="full"><span>Resposta observada</span><p>{detail.responseNotes || "—"}</p></div><div className="full"><span>Observações</span><p>{detail.notes || "—"}</p></div></div>}</Dialog>
    <Dialog open={Boolean(editForm)} onClose={() => !busy && setEditForm(null)} title="Corrigir alimentação" description="A correção mantém o mesmo registro e grava antes/depois na auditoria." size="large">{editForm && <form className="form-grid" onSubmit={submitCorrection}><label className="field"><span>Data e hora</span><input autoFocus required type="datetime-local" value={editForm.fedAt} onChange={(e) => setEditForm({ ...editForm, fedAt: e.target.value })} /></label><label className="field"><span>Tipo de alimento</span><input required value={editForm.foodType} onChange={(e) => setEditForm({ ...editForm, foodType: e.target.value })} /></label><label className="field"><span>Quantidade</span><input min="0" step="any" type="number" value={editForm.quantity ?? ""} onChange={(e) => setEditForm({ ...editForm, quantity: e.target.value ? Number(e.target.value) : undefined })} /></label><label className="field"><span>Unidade</span><input value={editForm.unit || ""} onChange={(e) => setEditForm({ ...editForm, unit: e.target.value || undefined })} /></label><label className="field full"><span>Resposta observada</span><input value={editForm.responseNotes || ""} onChange={(e) => setEditForm({ ...editForm, responseNotes: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={3} value={editForm.notes || ""} onChange={(e) => setEditForm({ ...editForm, notes: e.target.value })} /></label><label className="field"><span>Próxima alimentação</span><input type="datetime-local" value={editForm.nextFeedingAt || ""} onChange={(e) => setEditForm({ ...editForm, nextFeedingAt: e.target.value })} /></label><label className="field full"><span>Motivo da correção</span><textarea required rows={3} value={editForm.reason} onChange={(e) => setEditForm({ ...editForm, reason: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setEditForm(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !editForm.reason.trim() || !editForm.foodType.trim()}>Salvar correção</button></div></form>}</Dialog>
    <ReasonDialog open={Boolean(voidTarget)} title="Anular alimentação" description={voidTarget ? `${voidTarget.colonyCode} · ${formatDateTime(voidTarget.fedAt)}` : ""} confirmLabel="Anular registro" consequence="O registro continuará no histórico, mas deixará de representar alimentação válida e não gerará próxima alimentação pendente." danger busy={busy} onClose={() => setVoidTarget(null)} onConfirm={async (reason) => { if (!voidTarget) return false; const ok = await onVoid({ id: voidTarget.id, reason }); if (ok) await reload(); return ok; }} />
  </div>;
}

function FeedingCreateForm({ form, setForm, colonies, busy, onSubmit }: { form: FeedingForm; setForm: (value: FeedingForm) => void; colonies: Colony[]; busy: boolean; onSubmit: (event: FormEvent) => void }) { return <form className="form-grid" onSubmit={onSubmit}><label className="field full"><span>Colônia</span><select autoFocus required value={form.colonyId} onChange={(e) => setForm({ ...form, colonyId: e.target.value })}><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>)}</select></label><label className="field"><span>Data e hora</span><input type="datetime-local" value={form.fedAt} onChange={(e) => setForm({ ...form, fedAt: e.target.value })} /></label><label className="field"><span>Tipo de alimento</span><input required value={form.foodType} onChange={(e) => setForm({ ...form, foodType: e.target.value })} placeholder="Ex.: xarope 1:1" /></label><label className="field"><span>Quantidade</span><input min="0" step="any" type="number" value={form.quantity} onChange={(e) => setForm({ ...form, quantity: e.target.value })} /></label><label className="field"><span>Unidade</span><input value={form.unit} onChange={(e) => setForm({ ...form, unit: e.target.value })} placeholder="Ex.: ml" /></label><label className="field full"><span>Resposta observada</span><input value={form.responseNotes} onChange={(e) => setForm({ ...form, responseNotes: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={3} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} /></label><label className="field full"><span>Próxima alimentação</span><input type="datetime-local" value={form.nextFeedingAt} onChange={(e) => setForm({ ...form, nextFeedingAt: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setForm(initialForm)} disabled={busy}>Limpar</button><button type="submit" disabled={busy || !form.colonyId || !form.foodType.trim()}>{busy ? "Salvando..." : "Registrar alimentação"}</button></div></form>; }
function normalizeDateTime(value?: string) { if (!value) return undefined; const normalized = value.replace("T", " "); return normalized.length === 16 ? `${normalized}:00` : normalized; }
function toInputDateTime(value: string) { return value.replace(" ", "T").slice(0, 16); }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function quantityLabel(item: Feeding) { if (item.quantity == null || !item.unit) return "Sem quantidade"; return `${item.quantity} ${item.unit}`; }
