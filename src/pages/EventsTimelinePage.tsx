import { useEffect, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { ReasonDialog } from "../components/ReasonDialog";
import { RecordActions } from "../components/RecordActions";
import { RecordStateBadge } from "../components/RecordStateBadge";
import type { RecordStateMap } from "../hooks/useAppData";
import { getColonyTimeline, listColonyEvents } from "../lib/api";
import type { Colony, ColonyEvent, CorrectEventInput, CreateColonyEventInput, TimelineEntry, VoidRecordInput } from "../types";

type Props = {
  colonies: Colony[];
  busy: boolean;
  recordStateMap: RecordStateMap;
  onCreate: (input: CreateColonyEventInput) => Promise<boolean>;
  onCorrect: (input: CorrectEventInput) => Promise<boolean>;
  onVoid: (input: VoidRecordInput) => Promise<boolean>;
};
const initialForm: CreateColonyEventInput = { colonyId: "", eventType: "observation", occurredAt: "", title: "", details: "", severity: "info" };
const eventTypes = [["swarming", "Enxameação"], ["abandonment", "Abandono"], ["queen_loss", "Perda de rainha"], ["attack", "Ataque"], ["pest", "Praga ou inimigo"], ["recovery", "Recuperação"], ["maintenance", "Manutenção da colônia"], ["observation", "Observação"], ["other", "Outro evento"]] as const;

export function EventsTimelinePage({ colonies, busy, recordStateMap, onCreate, onCorrect, onVoid }: Props) {
  const [selectedColonyId, setSelectedColonyId] = useState("");
  const [form, setForm] = useState<CreateColonyEventInput>(initialForm);
  const [items, setItems] = useState<TimelineEntry[]>([]);
  const [events, setEvents] = useState<ColonyEvent[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadError, setLoadError] = useState("");
  const [open, setOpen] = useState(false);
  const [detail, setDetail] = useState<ColonyEvent | null>(null);
  const [editForm, setEditForm] = useState<CorrectEventInput | null>(null);
  const [voidTarget, setVoidTarget] = useState<ColonyEvent | null>(null);

  useEffect(() => { void reload(selectedColonyId); }, [selectedColonyId]);
  async function reload(colonyId = selectedColonyId) {
    if (!colonyId) { setItems([]); setEvents([]); setLoadError(""); return; }
    setLoading(true); setLoadError("");
    try { const [timeline, manualEvents] = await Promise.all([getColonyTimeline(colonyId), listColonyEvents(colonyId)]); setItems(timeline); setEvents(manualEvents); }
    catch { setLoadError("Não foi possível carregar o histórico desta colônia."); }
    finally { setLoading(false); }
  }
  function openNew() { setForm({ ...initialForm, colonyId: selectedColonyId }); setOpen(true); }
  async function submit(event: FormEvent) { event.preventDefault(); const input = { ...form, occurredAt: normalizeDateTime(form.occurredAt) }; if (await onCreate(input)) { setSelectedColonyId(form.colonyId); setOpen(false); setForm(initialForm); await reload(form.colonyId); } }
  function beginCorrect(item: ColonyEvent) { setEditForm({ id: item.id, reason: "", eventType: item.eventType, occurredAt: toInputDateTime(item.occurredAt), title: item.title || "", details: item.details || "", severity: item.severity }); }
  async function submitCorrection(event: FormEvent) { event.preventDefault(); if (!editForm) return; const payload: CorrectEventInput = { ...editForm, occurredAt: normalizeDateTime(editForm.occurredAt) || editForm.occurredAt }; if (await onCorrect(payload)) { setEditForm(null); await reload(); } }

  return <div className="page-stack">
    <PageToolbar title="Histórico" description="Timeline unificada de fatos operacionais da colônia." count={selectedColonyId ? `${items.length} fatos` : `${colonies.length} colônias`} primaryAction={{ label: "Novo evento", onClick: openNew, disabled: busy || colonies.length === 0 }}><label className="toolbar-select"><span className="sr-only">Colônia</span><select value={selectedColonyId} onChange={(e) => setSelectedColonyId(e.target.value)}><option value="">Selecione uma colônia...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>)}</select></label></PageToolbar>
    {loadError && <div className="inline-notice">{loadError}</div>}
    {colonies.length === 0 ? <div className="inline-notice">Cadastre uma colônia antes de registrar eventos.</div> : <div className="content-grid">
      <section className="panel wide-list"><div className="panel-heading"><h2>Histórico unificado</h2><p>Fatos válidos e marcadores administrativos aparecem em ordem cronológica.</p></div>{!selectedColonyId ? <div className="empty-list">Selecione uma colônia na toolbar para consultar a timeline.</div> : loading ? <div className="empty-list">Carregando histórico...</div> : items.length === 0 ? <div className="empty-list">Nenhum fato histórico encontrado.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Data</th><th>Tipo</th><th>Fato</th><th>Caixa</th><th>Importância</th><th>Detalhes</th></tr></thead><tbody>{items.map((item) => <tr key={`${item.sourceType}-${item.sourceId}`}><td><strong>{formatDateTime(item.occurredAt)}</strong></td><td>{sourceLabel(item.sourceType)}</td><td>{item.title}</td><td>{item.boxCode || "—"}</td><td><span className={`badge severity-${item.severity}`}>{severityLabel(item.severity)}</span></td><td>{item.details || "—"}</td></tr>)}</tbody></table></div>}</section>
      <section className="panel wide-list"><div className="panel-heading"><h2>Eventos manuais</h2><p>Eventos corrigidos ou anulados preservam a trilha administrativa sem reescrever outros tipos de fato.</p></div>{!selectedColonyId ? <div className="empty-list">Selecione uma colônia.</div> : events.length === 0 ? <div className="empty-list">Nenhum evento manual registrado.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Data</th><th>Evento</th><th>Estado</th><th>Ações</th></tr></thead><tbody>{events.map((item) => { const state = recordStateMap.get(`colony_event:${item.id}`); return <tr key={item.id} className={state?.voidedAt ? "voided-row" : undefined}><td><strong>{formatDateTime(item.occurredAt)}</strong></td><td>{item.title || eventLabel(item.eventType)}</td><td><RecordStateBadge state={state} /></td><td><RecordActions busy={busy} onOpen={() => setDetail(item)} onEdit={state?.voidedAt ? undefined : () => beginCorrect(item)} secondary={[{ label: "Anular", onClick: () => setVoidTarget(item), disabled: Boolean(state?.voidedAt), danger: true }]} /></td></tr>; })}</tbody></table></div>}</section>
    </div>}

    <Dialog open={open} onClose={() => !busy && setOpen(false)} title="Novo evento" description="Use eventos para fatos que não pertencem a outro registro operacional específico." size="medium"><EventCreateForm form={form} setForm={setForm} colonies={colonies} busy={busy} onSubmit={submit} /></Dialog>
    <Dialog open={Boolean(detail)} onClose={() => setDetail(null)} title="Evento" description={detail ? `${detail.colonyCode} · ${formatDateTime(detail.occurredAt)}` : ""} size="medium">{detail && <div className="detail-grid"><div><span>Tipo</span><strong>{eventLabel(detail.eventType)}</strong></div><div><span>Importância</span><strong>{severityLabel(detail.severity)}</strong></div><div><span>Caixa</span><strong>{detail.boxCode || "—"}</strong></div><div><span>Título</span><strong>{detail.title || "—"}</strong></div><div className="full"><span>Detalhes</span><p>{detail.details || "—"}</p></div></div>}</Dialog>
    <Dialog open={Boolean(editForm)} onClose={() => !busy && setEditForm(null)} title="Corrigir evento" description="Eventos representam fatos ocorridos. A correção fica registrada na auditoria." size="medium">{editForm && <form className="form-grid" onSubmit={submitCorrection}><label className="field"><span>Tipo</span><select value={editForm.eventType} onChange={(e) => setEditForm({ ...editForm, eventType: e.target.value })}>{eventTypes.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label><label className="field"><span>Importância</span><select value={editForm.severity} onChange={(e) => setEditForm({ ...editForm, severity: e.target.value })}><option value="info">Informativo</option><option value="attention">Atenção</option><option value="critical">Crítico</option></select></label><label className="field full"><span>Data e hora</span><input autoFocus required type="datetime-local" value={editForm.occurredAt} onChange={(e) => setEditForm({ ...editForm, occurredAt: e.target.value })} /></label><label className="field full"><span>Título</span><input value={editForm.title || ""} onChange={(e) => setEditForm({ ...editForm, title: e.target.value })} /></label><label className="field full"><span>Detalhes</span><textarea rows={4} value={editForm.details || ""} onChange={(e) => setEditForm({ ...editForm, details: e.target.value })} /></label><label className="field full"><span>Motivo da correção</span><textarea required rows={3} value={editForm.reason} onChange={(e) => setEditForm({ ...editForm, reason: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setEditForm(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !editForm.reason.trim()}>Salvar correção</button></div></form>}</Dialog>
    <ReasonDialog open={Boolean(voidTarget)} title="Anular evento" description={voidTarget ? `${voidTarget.colonyCode} · ${formatDateTime(voidTarget.occurredAt)}` : ""} confirmLabel="Anular registro" consequence="O evento continuará auditável, mas deixará de representar um fato operacional válido na timeline." danger busy={busy} onClose={() => setVoidTarget(null)} onConfirm={async (reason) => { if (!voidTarget) return false; const ok = await onVoid({ id: voidTarget.id, reason }); if (ok) await reload(); return ok; }} />
  </div>;
}

function EventCreateForm({ form, setForm, colonies, busy, onSubmit }: { form: CreateColonyEventInput; setForm: (value: CreateColonyEventInput) => void; colonies: Colony[]; busy: boolean; onSubmit: (event: FormEvent) => void }) { return <form className="form-grid" onSubmit={onSubmit}><label className="field full"><span>Colônia</span><select autoFocus required value={form.colonyId} onChange={(e) => setForm({ ...form, colonyId: e.target.value })}><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>)}</select></label><label className="field"><span>Tipo</span><select value={form.eventType} onChange={(e) => setForm({ ...form, eventType: e.target.value })}>{eventTypes.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label><label className="field"><span>Importância</span><select value={form.severity} onChange={(e) => setForm({ ...form, severity: e.target.value })}><option value="info">Informativo</option><option value="attention">Atenção</option><option value="critical">Crítico</option></select></label><label className="field full"><span>Data e hora</span><input type="datetime-local" value={form.occurredAt} onChange={(e) => setForm({ ...form, occurredAt: e.target.value })} /></label><label className="field full"><span>Título opcional</span><input value={form.title} onChange={(e) => setForm({ ...form, title: e.target.value })} /></label><label className="field full"><span>Detalhes</span><textarea rows={4} value={form.details} onChange={(e) => setForm({ ...form, details: e.target.value })} /></label><div className="form-actions full"><button type="submit" disabled={busy || !form.colonyId}>{busy ? "Salvando..." : "Registrar evento"}</button></div></form>; }
function normalizeDateTime(value?: string) { if (!value) return undefined; const normalized = value.replace("T", " "); return normalized.length === 16 ? `${normalized}:00` : normalized; }
function toInputDateTime(value: string) { return value.replace(" ", "T").slice(0, 16); }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function severityLabel(value: string) { return value === "critical" ? "Crítico" : value === "attention" ? "Atenção" : "Informativo"; }
function eventLabel(value: string) { return eventTypes.find(([key]) => key === value)?.[1] || value; }
function sourceLabel(value: string) { const labels: Record<string, string> = { event: "Evento", inspection: "Inspeção", feeding: "Alimentação", production: "Produção", movement: "Movimentação", box_occupancy: "Caixa", box_maintenance: "Manutenção", lifecycle: "Ciclo de vida", correction: "Correção", void: "Anulação", reversal: "Reversão" }; return labels[value] || value; }
