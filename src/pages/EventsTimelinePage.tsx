import { useEffect, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { getColonyTimeline } from "../lib/api";
import type { Colony, CreateColonyEventInput, TimelineEntry } from "../types";

type EventsTimelinePageProps = { colonies: Colony[]; busy: boolean; onCreate: (input: CreateColonyEventInput) => Promise<boolean>; };
const initialForm: CreateColonyEventInput = { colonyId: "", eventType: "observation", occurredAt: "", title: "", details: "", severity: "info" };
const eventTypes = [["swarming", "Enxameação"], ["abandonment", "Abandono"], ["queen_loss", "Perda de rainha"], ["attack", "Ataque"], ["pest", "Praga ou inimigo"], ["recovery", "Recuperação"], ["maintenance", "Manutenção da colônia"], ["observation", "Observação"], ["other", "Outro evento"]] as const;

export function EventsTimelinePage({ colonies, busy, onCreate }: EventsTimelinePageProps) {
  const [selectedColonyId, setSelectedColonyId] = useState("");
  const [form, setForm] = useState<CreateColonyEventInput>(initialForm);
  const [items, setItems] = useState<TimelineEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadError, setLoadError] = useState("");
  const [open, setOpen] = useState(false);

  useEffect(() => {
    let cancelled = false;
    if (!selectedColonyId) { setItems([]); setLoadError(""); return; }
    setLoading(true); setLoadError("");
    getColonyTimeline(selectedColonyId).then((data) => { if (!cancelled) setItems(data); }).catch(() => { if (!cancelled) setLoadError("Não foi possível carregar a timeline desta colônia."); }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [selectedColonyId]);

  async function reload(colonyId = selectedColonyId) { if (colonyId) setItems(await getColonyTimeline(colonyId)); }
  function openNew() { setForm({ ...initialForm, colonyId: selectedColonyId }); setOpen(true); }
  async function submit(event: FormEvent) { event.preventDefault(); const input = { ...form, occurredAt: normalizeDateTime(form.occurredAt) }; if (await onCreate(input)) { setSelectedColonyId(form.colonyId); setOpen(false); setForm(initialForm); await reload(form.colonyId); } }

  return <div className="page-stack">
    <PageToolbar title="Histórico" description="Timeline unificada de fatos operacionais da colônia." count={selectedColonyId ? `${items.length} fatos` : `${colonies.length} colônias`} primaryAction={{ label: "Novo evento", onClick: openNew, disabled: busy || colonies.length === 0 }}><label className="toolbar-select"><span className="sr-only">Colônia</span><select value={selectedColonyId} onChange={(e) => setSelectedColonyId(e.target.value)}><option value="">Selecione uma colônia...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>)}</select></label></PageToolbar>
    {colonies.length === 0 ? <div className="inline-notice">Cadastre uma colônia antes de registrar eventos.</div> : <section className="panel wide-list"><div className="panel-heading"><h2>Histórico unificado</h2><p>Inspeções, alimentação, produção, movimentações, ocupações, manutenção, ciclo de vida e eventos em ordem cronológica.</p></div>{!selectedColonyId ? <div className="empty-list">Selecione uma colônia na toolbar para consultar a timeline.</div> : loading ? <div className="empty-list">Carregando histórico...</div> : loadError ? <div className="inline-notice">{loadError}</div> : items.length === 0 ? <div className="empty-list">Nenhum fato histórico encontrado.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Data</th><th>Tipo</th><th>Fato</th><th>Caixa</th><th>Importância</th><th>Detalhes</th></tr></thead><tbody>{items.map((item) => <tr key={`${item.sourceType}-${item.sourceId}`}><td><strong>{formatDateTime(item.occurredAt)}</strong></td><td>{sourceLabel(item.sourceType)}</td><td>{item.title}</td><td>{item.boxCode || "—"}</td><td><span className={`badge severity-${item.severity}`}>{severityLabel(item.severity)}</span></td><td>{item.details || "—"}</td></tr>)}</tbody></table></div>}</section>}
    <Dialog open={open} onClose={() => !busy && setOpen(false)} title="Novo evento" description="Use eventos para fatos que não pertencem a outro registro operacional específico." size="medium"><form className="form-grid" onSubmit={submit}><label className="field full"><span>Colônia</span><select autoFocus required value={form.colonyId} onChange={(e) => setForm({ ...form, colonyId: e.target.value })}><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>)}</select></label><label className="field"><span>Tipo</span><select value={form.eventType} onChange={(e) => setForm({ ...form, eventType: e.target.value })}>{eventTypes.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label><label className="field"><span>Importância</span><select value={form.severity} onChange={(e) => setForm({ ...form, severity: e.target.value })}><option value="info">Informativo</option><option value="attention">Atenção</option><option value="critical">Crítico</option></select></label><label className="field full"><span>Data e hora</span><input type="datetime-local" value={form.occurredAt} onChange={(e) => setForm({ ...form, occurredAt: e.target.value })} /></label><label className="field full"><span>Título opcional</span><input value={form.title} onChange={(e) => setForm({ ...form, title: e.target.value })} /></label><label className="field full"><span>Detalhes</span><textarea rows={4} value={form.details} onChange={(e) => setForm({ ...form, details: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setOpen(false)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !form.colonyId}>{busy ? "Salvando..." : "Registrar evento"}</button></div></form></Dialog>
  </div>;
}
function normalizeDateTime(value?: string) { if (!value) return undefined; const normalized = value.replace("T", " "); return normalized.length === 16 ? `${normalized}:00` : normalized; }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function severityLabel(value: string) { return value === "critical" ? "Crítico" : value === "attention" ? "Atenção" : "Informativo"; }
function sourceLabel(value: string) { const labels: Record<string, string> = { event: "Evento", inspection: "Inspeção", feeding: "Alimentação", production: "Produção", movement: "Movimentação", box_occupancy: "Caixa", box_maintenance: "Manutenção", lifecycle: "Ciclo de vida" }; return labels[value] || value; }
