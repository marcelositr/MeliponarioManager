import { useEffect, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { listColonyProduction } from "../lib/api";
import type { Colony, CreateProductionInput, ProductionRecord } from "../types";

type ProductionPageProps = { colonies: Colony[]; busy: boolean; onCreate: (input: CreateProductionInput) => Promise<boolean>; };
type ProductionForm = { colonyId: string; harvestedAt: string; productType: string; quantity: string; unit: string; purpose: string; notes: string; };
const initialForm: ProductionForm = { colonyId: "", harvestedAt: "", productType: "honey", quantity: "", unit: "ml", purpose: "", notes: "" };
const productOptions = [["honey", "Mel"], ["pollen", "Pólen"], ["propolis", "Própolis"], ["wax", "Cera"], ["cerumen", "Cerume"], ["other", "Outro produto"]] as const;

export function ProductionPage({ colonies, busy, onCreate }: ProductionPageProps) {
  const [selectedColonyId, setSelectedColonyId] = useState("");
  const [form, setForm] = useState<ProductionForm>(initialForm);
  const [items, setItems] = useState<ProductionRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadError, setLoadError] = useState("");
  const [open, setOpen] = useState(false);

  useEffect(() => {
    let cancelled = false;
    if (!selectedColonyId) { setItems([]); setLoadError(""); return; }
    setLoading(true); setLoadError("");
    listColonyProduction(selectedColonyId).then((data) => { if (!cancelled) setItems(data); }).catch(() => { if (!cancelled) setLoadError("Não foi possível carregar a produção desta colônia."); }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [selectedColonyId]);

  async function reload(colonyId = selectedColonyId) { if (colonyId) setItems(await listColonyProduction(colonyId)); }
  function openNew() { setForm({ ...initialForm, colonyId: selectedColonyId }); setOpen(true); }
  async function submit(event: FormEvent) {
    event.preventDefault();
    const input: CreateProductionInput = { colonyId: form.colonyId, harvestedAt: normalizeDateTime(form.harvestedAt), productType: form.productType, quantity: Number(form.quantity), unit: form.unit, purpose: form.purpose, notes: form.notes };
    if (await onCreate(input)) { setSelectedColonyId(form.colonyId); setOpen(false); const colonyId = form.colonyId; const productType = form.productType; const unit = form.unit; setForm({ ...initialForm, colonyId, productType, unit }); await reload(colonyId); }
  }

  return <div className="page-stack">
    <PageToolbar title="Produção" description="Colheitas quantificadas por colônia." count={selectedColonyId ? `${items.length} registros` : `${colonies.length} colônias`} primaryAction={{ label: "Nova produção", onClick: openNew, disabled: busy || colonies.length === 0 }}><label className="toolbar-select"><span className="sr-only">Colônia</span><select value={selectedColonyId} onChange={(e) => setSelectedColonyId(e.target.value)}><option value="">Selecione uma colônia...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>)}</select></label></PageToolbar>
    {colonies.length === 0 ? <div className="inline-notice">Cadastre uma colônia antes de registrar produção.</div> : <section className="panel wide-list"><div className="panel-heading"><h2>Histórico da colônia</h2><p>A caixa exibida corresponde à ocupação existente na data da colheita.</p></div>{!selectedColonyId ? <div className="empty-list">Selecione uma colônia na toolbar para consultar o histórico.</div> : loading ? <div className="empty-list">Carregando produção...</div> : loadError ? <div className="inline-notice">{loadError}</div> : items.length === 0 ? <div className="empty-list">Nenhuma produção registrada para esta colônia.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Data</th><th>Produto</th><th>Quantidade</th><th>Caixa</th><th>Finalidade</th></tr></thead><tbody>{items.map((item) => <tr key={item.id}><td><strong>{formatDateTime(item.harvestedAt)}</strong></td><td>{productLabel(item.productType)}</td><td>{item.quantity} {item.unit}</td><td>{item.boxCode || "Sem caixa"}</td><td>{item.purpose || "—"}</td></tr>)}</tbody></table></div>}</section>}
    <Dialog open={open} onClose={() => !busy && setOpen(false)} title="Nova produção" description="Produto, quantidade e unidade permanecem obrigatórios para manter registros agregáveis." size="medium"><form className="form-grid" onSubmit={submit}><label className="field full"><span>Colônia</span><select autoFocus required value={form.colonyId} onChange={(e) => setForm({ ...form, colonyId: e.target.value })}><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>)}</select></label><label className="field"><span>Data e hora</span><input type="datetime-local" value={form.harvestedAt} onChange={(e) => setForm({ ...form, harvestedAt: e.target.value })} /></label><label className="field"><span>Produto</span><select value={form.productType} onChange={(e) => setForm({ ...form, productType: e.target.value })}>{productOptions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label><label className="field"><span>Quantidade</span><input required min="0.000001" step="any" type="number" value={form.quantity} onChange={(e) => setForm({ ...form, quantity: e.target.value })} /></label><label className="field"><span>Unidade</span><input required value={form.unit} onChange={(e) => setForm({ ...form, unit: e.target.value })} /></label><label className="field full"><span>Destino ou finalidade</span><input value={form.purpose} onChange={(e) => setForm({ ...form, purpose: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={3} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setOpen(false)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !form.colonyId || !form.quantity || !form.unit.trim()}>{busy ? "Salvando..." : "Registrar produção"}</button></div></form></Dialog>
  </div>;
}
function normalizeDateTime(value?: string) { if (!value) return undefined; const normalized = value.replace("T", " "); return normalized.length === 16 ? `${normalized}:00` : normalized; }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function productLabel(value: string) { return productOptions.find(([key]) => key === value)?.[1] || value; }
