import { useEffect, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { ReasonDialog } from "../components/ReasonDialog";
import { RecordActions } from "../components/RecordActions";
import { RecordStateBadge } from "../components/RecordStateBadge";
import type { RecordStateMap } from "../hooks/useAppData";
import { listColonyProduction } from "../lib/api";
import type { Colony, CorrectProductionInput, CreateProductionInput, ProductionRecord, VoidRecordInput } from "../types";

type Props = {
  colonies: Colony[];
  busy: boolean;
  recordStateMap: RecordStateMap;
  onCreate: (input: CreateProductionInput) => Promise<boolean>;
  onCorrect: (input: CorrectProductionInput) => Promise<boolean>;
  onVoid: (input: VoidRecordInput) => Promise<boolean>;
};
type ProductionForm = { colonyId: string; harvestedAt: string; productType: string; quantity: string; unit: string; purpose: string; notes: string };
const initialForm: ProductionForm = { colonyId: "", harvestedAt: "", productType: "honey", quantity: "", unit: "ml", purpose: "", notes: "" };
const productOptions = [["honey", "Mel"], ["pollen", "Pólen"], ["propolis", "Própolis"], ["wax", "Cera"], ["cerumen", "Cerume"], ["other", "Outro produto"]] as const;

export function ProductionPage({ colonies, busy, recordStateMap, onCreate, onCorrect, onVoid }: Props) {
  const [selectedColonyId, setSelectedColonyId] = useState("");
  const [form, setForm] = useState<ProductionForm>(initialForm);
  const [items, setItems] = useState<ProductionRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadError, setLoadError] = useState("");
  const [open, setOpen] = useState(false);
  const [detail, setDetail] = useState<ProductionRecord | null>(null);
  const [editForm, setEditForm] = useState<CorrectProductionInput | null>(null);
  const [voidTarget, setVoidTarget] = useState<ProductionRecord | null>(null);

  useEffect(() => { void reload(selectedColonyId); }, [selectedColonyId]);
  async function reload(colonyId = selectedColonyId) {
    if (!colonyId) { setItems([]); setLoadError(""); return; }
    setLoading(true); setLoadError("");
    try { setItems(await listColonyProduction(colonyId)); }
    catch { setLoadError("Não foi possível carregar a produção desta colônia."); }
    finally { setLoading(false); }
  }
  function openNew() { setForm({ ...initialForm, colonyId: selectedColonyId }); setOpen(true); }
  async function submit(event: FormEvent) {
    event.preventDefault();
    const input: CreateProductionInput = { colonyId: form.colonyId, harvestedAt: normalizeDateTime(form.harvestedAt), productType: form.productType, quantity: Number(form.quantity), unit: form.unit, purpose: form.purpose, notes: form.notes };
    if (await onCreate(input)) { setSelectedColonyId(form.colonyId); setOpen(false); const colonyId = form.colonyId; const productType = form.productType; const unit = form.unit; setForm({ ...initialForm, colonyId, productType, unit }); await reload(colonyId); }
  }
  function beginCorrect(item: ProductionRecord) {
    setEditForm({ id: item.id, reason: "", harvestedAt: toInputDateTime(item.harvestedAt), productType: item.productType, quantity: item.quantity, unit: item.unit, purpose: item.purpose || "", notes: item.notes || "" });
  }
  async function submitCorrection(event: FormEvent) {
    event.preventDefault(); if (!editForm) return;
    const payload: CorrectProductionInput = { ...editForm, harvestedAt: normalizeDateTime(editForm.harvestedAt) || editForm.harvestedAt };
    if (await onCorrect(payload)) { setEditForm(null); await reload(); }
  }

  return <div className="page-stack">
    <PageToolbar title="Produção" description="Colheitas quantificadas por colônia." count={selectedColonyId ? `${items.length} registros` : `${colonies.length} colônias`} primaryAction={{ label: "Nova produção", onClick: openNew, disabled: busy || colonies.length === 0 }}><label className="toolbar-select"><span className="sr-only">Colônia</span><select value={selectedColonyId} onChange={(e) => setSelectedColonyId(e.target.value)}><option value="">Selecione uma colônia...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>)}</select></label></PageToolbar>
    {colonies.length === 0 ? <div className="inline-notice">Cadastre uma colônia antes de registrar produção.</div> : <section className="panel wide-list"><div className="panel-heading"><h2>Histórico da colônia</h2><p>Produções anuladas permanecem rastreáveis e deixam de participar dos totais válidos.</p></div>{!selectedColonyId ? <div className="empty-list">Selecione uma colônia na toolbar para consultar o histórico.</div> : loading ? <div className="empty-list">Carregando produção...</div> : loadError ? <div className="inline-notice">{loadError}</div> : items.length === 0 ? <div className="empty-list">Nenhuma produção registrada para esta colônia.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Data</th><th>Produto</th><th>Quantidade</th><th>Caixa</th><th>Estado</th><th>Ações</th></tr></thead><tbody>{items.map((item) => { const state = recordStateMap.get(`production:${item.id}`); return <tr key={item.id} className={state?.voidedAt ? "voided-row" : undefined}><td><strong>{formatDateTime(item.harvestedAt)}</strong></td><td>{productLabel(item.productType)}</td><td>{item.quantity} {item.unit}</td><td>{item.boxCode || "Sem caixa"}</td><td><RecordStateBadge state={state} /></td><td><RecordActions busy={busy} onOpen={() => setDetail(item)} onEdit={state?.voidedAt ? undefined : () => beginCorrect(item)} secondary={[{ label: "Anular", onClick: () => setVoidTarget(item), disabled: Boolean(state?.voidedAt), danger: true }]} /></td></tr>; })}</tbody></table></div>}</section>}

    <Dialog open={open} onClose={() => !busy && setOpen(false)} title="Nova produção" description="Produto, quantidade e unidade permanecem obrigatórios para manter registros agregáveis." size="medium"><ProductionCreateForm form={form} setForm={setForm} colonies={colonies} busy={busy} onSubmit={submit} /></Dialog>
    <Dialog open={Boolean(detail)} onClose={() => setDetail(null)} title="Produção" description={detail ? `${detail.colonyCode} · ${formatDateTime(detail.harvestedAt)}` : ""} size="medium">{detail && <div className="detail-grid"><div><span>Produto</span><strong>{productLabel(detail.productType)}</strong></div><div><span>Quantidade</span><strong>{detail.quantity} {detail.unit}</strong></div><div><span>Caixa</span><strong>{detail.boxCode || "Sem caixa"}</strong></div><div><span>Finalidade</span><strong>{detail.purpose || "—"}</strong></div><div className="full"><span>Observações</span><p>{detail.notes || "—"}</p></div></div>}</Dialog>
    <Dialog open={Boolean(editForm)} onClose={() => !busy && setEditForm(null)} title="Corrigir produção" description="A correção preserva o valor anterior na auditoria e recalcula a projeção válida." size="medium">{editForm && <form className="form-grid" onSubmit={submitCorrection}><label className="field"><span>Data e hora</span><input autoFocus required type="datetime-local" value={editForm.harvestedAt} onChange={(e) => setEditForm({ ...editForm, harvestedAt: e.target.value })} /></label><label className="field"><span>Produto</span><select value={editForm.productType} onChange={(e) => setEditForm({ ...editForm, productType: e.target.value })}>{productOptions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label><label className="field"><span>Quantidade</span><input required min="0.000001" step="any" type="number" value={editForm.quantity} onChange={(e) => setEditForm({ ...editForm, quantity: Number(e.target.value) })} /></label><label className="field"><span>Unidade</span><input required value={editForm.unit} onChange={(e) => setEditForm({ ...editForm, unit: e.target.value })} /></label><label className="field full"><span>Destino ou finalidade</span><input value={editForm.purpose || ""} onChange={(e) => setEditForm({ ...editForm, purpose: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={3} value={editForm.notes || ""} onChange={(e) => setEditForm({ ...editForm, notes: e.target.value })} /></label><label className="field full"><span>Motivo da correção</span><textarea required rows={3} value={editForm.reason} onChange={(e) => setEditForm({ ...editForm, reason: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setEditForm(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !editForm.reason.trim() || editForm.quantity <= 0 || !editForm.unit.trim()}>Salvar correção</button></div></form>}</Dialog>
    <ReasonDialog open={Boolean(voidTarget)} title="Anular produção" description={voidTarget ? `${voidTarget.colonyCode} · ${formatDateTime(voidTarget.harvestedAt)}` : ""} confirmLabel="Anular registro" consequence="A produção continuará no histórico para rastreabilidade, mas deixará de compor contagens e totais operacionais válidos." danger busy={busy} onClose={() => setVoidTarget(null)} onConfirm={async (reason) => { if (!voidTarget) return false; const ok = await onVoid({ id: voidTarget.id, reason }); if (ok) await reload(); return ok; }} />
  </div>;
}

function ProductionCreateForm({ form, setForm, colonies, busy, onSubmit }: { form: ProductionForm; setForm: (value: ProductionForm) => void; colonies: Colony[]; busy: boolean; onSubmit: (event: FormEvent) => void }) { return <form className="form-grid" onSubmit={onSubmit}><label className="field full"><span>Colônia</span><select autoFocus required value={form.colonyId} onChange={(e) => setForm({ ...form, colonyId: e.target.value })}><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>)}</select></label><label className="field"><span>Data e hora</span><input type="datetime-local" value={form.harvestedAt} onChange={(e) => setForm({ ...form, harvestedAt: e.target.value })} /></label><label className="field"><span>Produto</span><select value={form.productType} onChange={(e) => setForm({ ...form, productType: e.target.value })}>{productOptions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label><label className="field"><span>Quantidade</span><input required min="0.000001" step="any" type="number" value={form.quantity} onChange={(e) => setForm({ ...form, quantity: e.target.value })} /></label><label className="field"><span>Unidade</span><input required value={form.unit} onChange={(e) => setForm({ ...form, unit: e.target.value })} /></label><label className="field full"><span>Destino ou finalidade</span><input value={form.purpose} onChange={(e) => setForm({ ...form, purpose: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={3} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} /></label><div className="form-actions full"><button type="submit" disabled={busy || !form.colonyId || !form.quantity || !form.unit.trim()}>{busy ? "Salvando..." : "Registrar produção"}</button></div></form>; }
function normalizeDateTime(value?: string) { if (!value) return undefined; const normalized = value.replace("T", " "); return normalized.length === 16 ? `${normalized}:00` : normalized; }
function toInputDateTime(value: string) { return value.replace(" ", "T").slice(0, 16); }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function productLabel(value: string) { return productOptions.find(([key]) => key === value)?.[1] || value; }
