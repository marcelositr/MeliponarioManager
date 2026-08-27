import { useEffect, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { listColonyFeedings } from "../lib/api";
import type { Colony, CreateFeedingInput, Feeding } from "../types";

type FeedingPageProps = { colonies: Colony[]; busy: boolean; onCreate: (input: CreateFeedingInput) => Promise<boolean>; };
type FeedingForm = { colonyId: string; fedAt: string; foodType: string; quantity: string; unit: string; responseNotes: string; notes: string; nextFeedingAt: string; };
const initialForm: FeedingForm = { colonyId: "", fedAt: "", foodType: "", quantity: "", unit: "", responseNotes: "", notes: "", nextFeedingAt: "" };

export function FeedingPage({ colonies, busy, onCreate }: FeedingPageProps) {
  const [selectedColonyId, setSelectedColonyId] = useState("");
  const [form, setForm] = useState<FeedingForm>(initialForm);
  const [items, setItems] = useState<Feeding[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadError, setLoadError] = useState("");
  const [open, setOpen] = useState(false);

  useEffect(() => {
    let cancelled = false;
    if (!selectedColonyId) { setItems([]); setLoadError(""); return; }
    setLoading(true); setLoadError("");
    listColonyFeedings(selectedColonyId).then((data) => { if (!cancelled) setItems(data); }).catch(() => { if (!cancelled) setLoadError("Não foi possível carregar as alimentações desta colônia."); }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [selectedColonyId]);

  async function reload(colonyId = selectedColonyId) { if (colonyId) setItems(await listColonyFeedings(colonyId)); }
  function openNew() { setForm({ ...initialForm, colonyId: selectedColonyId }); setOpen(true); }
  async function submit(event: FormEvent) {
    event.preventDefault();
    const quantity = form.quantity.trim() ? Number(form.quantity) : undefined;
    const input: CreateFeedingInput = { colonyId: form.colonyId, fedAt: normalizeDateTime(form.fedAt), foodType: form.foodType, quantity, unit: form.unit.trim() || undefined, responseNotes: form.responseNotes, notes: form.notes, nextFeedingAt: normalizeDateTime(form.nextFeedingAt) };
    if (await onCreate(input)) { setSelectedColonyId(form.colonyId); setOpen(false); setForm(initialForm); await reload(form.colonyId); }
  }

  return <div className="page-stack">
    <PageToolbar title="Alimentação" description="Suplementações e resposta observada por colônia." count={selectedColonyId ? `${items.length} registros` : `${colonies.length} colônias`} primaryAction={{ label: "Nova alimentação", onClick: openNew, disabled: busy || colonies.length === 0 }}><label className="toolbar-select"><span className="sr-only">Colônia</span><select value={selectedColonyId} onChange={(e) => setSelectedColonyId(e.target.value)}><option value="">Selecione uma colônia...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>)}</select></label></PageToolbar>
    {colonies.length === 0 ? <div className="inline-notice">Cadastre uma colônia antes de registrar alimentação.</div> : <section className="panel wide-list"><div className="panel-heading"><h2>Histórico da colônia</h2><p>A caixa do registro é resolvida conforme a data informada.</p></div>{!selectedColonyId ? <div className="empty-list">Selecione uma colônia na toolbar para consultar o histórico.</div> : loading ? <div className="empty-list">Carregando alimentações...</div> : loadError ? <div className="inline-notice">{loadError}</div> : items.length === 0 ? <div className="empty-list">Nenhuma alimentação registrada para esta colônia.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Data</th><th>Alimento</th><th>Quantidade</th><th>Caixa</th><th>Resposta</th><th>Próxima</th></tr></thead><tbody>{items.map((item) => <tr key={item.id}><td><strong>{formatDateTime(item.fedAt)}</strong></td><td>{item.foodType}</td><td>{quantityLabel(item)}</td><td>{item.boxCode || "Sem caixa"}</td><td>{item.responseNotes || "—"}</td><td>{item.nextFeedingAt ? formatDateTime(item.nextFeedingAt) : "Sem agendamento"}</td></tr>)}</tbody></table></div>}</section>}
    <Dialog open={open} onClose={() => !busy && setOpen(false)} title="Nova alimentação" description="Quantidade e unidade são opcionais, mas devem ser coerentes quando informadas." size="large"><form className="form-grid" onSubmit={submit}><label className="field full"><span>Colônia</span><select autoFocus required value={form.colonyId} onChange={(e) => setForm({ ...form, colonyId: e.target.value })}><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>)}</select></label><label className="field"><span>Data e hora</span><input type="datetime-local" value={form.fedAt} onChange={(e) => setForm({ ...form, fedAt: e.target.value })} /></label><label className="field"><span>Tipo de alimento</span><input required value={form.foodType} onChange={(e) => setForm({ ...form, foodType: e.target.value })} placeholder="Ex.: xarope 1:1" /></label><label className="field"><span>Quantidade</span><input min="0" step="any" type="number" value={form.quantity} onChange={(e) => setForm({ ...form, quantity: e.target.value })} /></label><label className="field"><span>Unidade</span><input value={form.unit} onChange={(e) => setForm({ ...form, unit: e.target.value })} placeholder="Ex.: ml" /></label><label className="field full"><span>Resposta observada</span><input value={form.responseNotes} onChange={(e) => setForm({ ...form, responseNotes: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={3} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} /></label><label className="field full"><span>Próxima alimentação</span><input type="datetime-local" value={form.nextFeedingAt} onChange={(e) => setForm({ ...form, nextFeedingAt: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setOpen(false)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !form.colonyId || !form.foodType.trim()}>{busy ? "Salvando..." : "Registrar alimentação"}</button></div></form></Dialog>
  </div>;
}
function normalizeDateTime(value?: string) { if (!value) return undefined; const normalized = value.replace("T", " "); return normalized.length === 16 ? `${normalized}:00` : normalized; }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function quantityLabel(item: Feeding) { if (item.quantity == null || !item.unit) return "Sem quantidade"; return `${item.quantity} ${item.unit}`; }
