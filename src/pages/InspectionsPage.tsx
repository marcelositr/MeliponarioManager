import { useEffect, useRef, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { ReasonDialog } from "../components/ReasonDialog";
import { RecordActions } from "../components/RecordActions";
import { RecordStateBadge } from "../components/RecordStateBadge";
import type { RecordStateMap } from "../hooks/useAppData";
import { listColonyInspections } from "../lib/api";
import { formatDateTimeBr } from "../lib/presentation";
import type { Colony, CorrectInspectionInput, CreateInspectionInput, Inspection, VoidRecordInput } from "../types";

type Props = { colonies: Colony[]; busy: boolean; recordStateMap: RecordStateMap; autoCreate?: boolean; onCreate: (input: CreateInspectionInput) => Promise<boolean>; onCorrect: (input: CorrectInspectionInput) => Promise<boolean>; onVoid: (input: VoidRecordInput) => Promise<boolean> };
const initialForm: CreateInspectionInput = { colonyId: "", inspectedAt: "", strength: "unknown", layingStatus: "", foodReserves: "", broodStatus: "", pestsNotes: "", observations: "", actionsTaken: "", nextInspectionAt: "" };
const strengthOptions = [["strong", "Forte"], ["medium", "Média"], ["weak", "Fraca"], ["unknown", "Não avaliada"]] as const;

export function InspectionsPage({ colonies, busy, recordStateMap, autoCreate = false, onCreate, onCorrect, onVoid }: Props) {
  const [selectedColonyId, setSelectedColonyId] = useState("");
  const [form, setForm] = useState<CreateInspectionInput>(initialForm);
  const [queenValue, setQueenValue] = useState("");
  const [items, setItems] = useState<Inspection[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadError, setLoadError] = useState("");
  const [open, setOpen] = useState(false);
  const [detail, setDetail] = useState<Inspection | null>(null);
  const [editForm, setEditForm] = useState<CorrectInspectionInput | null>(null);
  const [editQueen, setEditQueen] = useState("");
  const [voidTarget, setVoidTarget] = useState<Inspection | null>(null);
  const handledAutoCreate = useRef(false);

  useEffect(() => { void reload(selectedColonyId); }, [selectedColonyId]);
  useEffect(() => {
    if (!autoCreate) { handledAutoCreate.current = false; return; }
    if (handledAutoCreate.current || colonies.length !== 1) return;
    handledAutoCreate.current = true;
    const colonyId = colonies[0].id;
    setSelectedColonyId(colonyId);
    setForm({ ...initialForm, colonyId });
    setQueenValue("");
    setOpen(true);
  }, [autoCreate, colonies]);

  async function reload(colonyId = selectedColonyId) {
    if (!colonyId) { setItems([]); setLoadError(""); return; }
    setLoading(true); setLoadError("");
    try { setItems(await listColonyInspections(colonyId)); } catch { setLoadError("Não foi possível carregar as inspeções desta colônia."); }
    finally { setLoading(false); }
  }
  function openNew() { setForm({ ...initialForm, colonyId: selectedColonyId }); setQueenValue(""); setOpen(true); }
  async function submit(event: FormEvent) {
    event.preventDefault();
    const input: CreateInspectionInput = { ...form, inspectedAt: normalizeDateTime(form.inspectedAt), nextInspectionAt: normalizeDateTime(form.nextInspectionAt), queenPresent: queenValue === "yes" ? true : queenValue === "no" ? false : null };
    if (await onCreate(input)) { setSelectedColonyId(form.colonyId); setOpen(false); setForm(initialForm); setQueenValue(""); await reload(form.colonyId); }
  }
  function beginCorrect(item: Inspection) {
    setEditQueen(item.queenPresent === true ? "yes" : item.queenPresent === false ? "no" : "");
    setEditForm({ id: item.id, reason: "", inspectedAt: toInputDateTime(item.inspectedAt), strength: item.strength, queenPresent: item.queenPresent, layingStatus: item.layingStatus || "", foodReserves: item.foodReserves || "", broodStatus: item.broodStatus || "", pestsNotes: item.pestsNotes || "", observations: item.observations || "", actionsTaken: item.actionsTaken || "", nextInspectionAt: item.nextInspectionAt ? toInputDateTime(item.nextInspectionAt) : "" });
  }
  async function submitCorrection(event: FormEvent) {
    event.preventDefault(); if (!editForm) return;
    const payload = { ...editForm, inspectedAt: normalizeDateTime(editForm.inspectedAt) || editForm.inspectedAt, nextInspectionAt: normalizeDateTime(editForm.nextInspectionAt), queenPresent: editQueen === "yes" ? true : editQueen === "no" ? false : null };
    if (await onCorrect(payload)) { setEditForm(null); await reload(); }
  }

  return <div className="page-stack">
    <PageToolbar title="Inspeções" description="Condição observada e histórico cronológico por colônia." count={selectedColonyId ? `${items.length} registros` : `${colonies.length} colônias`} primaryAction={{ label: "Nova inspeção", onClick: openNew, disabled: busy || colonies.length === 0 }}><label className="toolbar-select"><span className="sr-only">Colônia</span><select value={selectedColonyId} onChange={(e) => setSelectedColonyId(e.target.value)}><option value="">Selecione uma colônia...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>)}</select></label></PageToolbar>
    {colonies.length === 0 ? <div className="inline-notice" role="status">Cadastre uma colônia antes de registrar inspeções.</div> : <section className="panel wide-list"><div className="panel-heading"><h2>Histórico da colônia</h2><p>Correções preservam fotos e auditoria. Registros anulados continuam visíveis, mas deixam de alimentar força atual e alertas.</p></div>{!selectedColonyId ? <div className="empty-list">Selecione uma colônia na toolbar para consultar o histórico.</div> : loading ? <div className="empty-list" role="status">Carregando inspeções...</div> : loadError ? <div className="inline-notice" role="alert">{loadError}</div> : items.length === 0 ? <div className="empty-list">Nenhuma inspeção registrada para esta colônia.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Data</th><th>Caixa</th><th>Força</th><th>Rainha</th><th>Próxima</th><th>Estado</th><th>Ações</th></tr></thead><tbody>{items.map((item) => { const state = recordStateMap.get(`inspection:${item.id}`); return <tr key={item.id} className={state?.voidedAt ? "voided-row" : undefined}><td><strong>{formatDateTimeBr(item.inspectedAt)}</strong></td><td>{item.boxCode || "Sem caixa"}</td><td><span className={`badge status-${item.strength}`}>{strengthLabel(item.strength)}</span></td><td>{booleanLabel(item.queenPresent)}</td><td>{item.nextInspectionAt ? formatDateTimeBr(item.nextInspectionAt) : "Sem agendamento"}</td><td><RecordStateBadge state={state} /></td><td><RecordActions busy={busy} onOpen={() => setDetail(item)} onEdit={state?.voidedAt ? undefined : () => beginCorrect(item)} secondary={[{ label: "Anular", onClick: () => setVoidTarget(item), disabled: Boolean(state?.voidedAt), danger: true }]} /></td></tr>; })}</tbody></table></div>}</section>}

    <Dialog open={open} onClose={() => !busy && setOpen(false)} title="Nova inspeção" description="Registre somente o que foi observado; a força alimenta os alertas derivados." size="large"><InspectionForm form={form} setForm={setForm} queenValue={queenValue} setQueenValue={setQueenValue} colonies={colonies} busy={busy} onSubmit={submit} submitLabel="Registrar inspeção" /></Dialog>
    <Dialog open={Boolean(detail)} onClose={() => setDetail(null)} title="Inspeção" description={detail ? `${detail.colonyCode} · ${formatDateTimeBr(detail.inspectedAt)}` : ""} size="large">{detail && <div className="detail-grid"><div><span>Caixa</span><strong>{detail.boxCode || "Sem caixa"}</strong></div><div><span>Força</span><strong>{strengthLabel(detail.strength)}</strong></div><div><span>Rainha</span><strong>{booleanLabel(detail.queenPresent)}</strong></div><div><span>Postura</span><strong>{detail.layingStatus || "—"}</strong></div><div><span>Reservas</span><strong>{detail.foodReserves || "—"}</strong></div><div><span>Crias</span><strong>{detail.broodStatus || "—"}</strong></div><div className="full"><span>Pragas</span><p>{detail.pestsNotes || "—"}</p></div><div className="full"><span>Observações</span><p>{detail.observations || "—"}</p></div><div className="full"><span>Ações realizadas</span><p>{detail.actionsTaken || "—"}</p></div></div>}</Dialog>
    <Dialog open={Boolean(editForm)} onClose={() => !busy && setEditForm(null)} title="Corrigir inspeção" description="A versão anterior fica registrada na auditoria; fotos permanecem ligadas à mesma inspeção." size="large">{editForm && <form className="form-grid" onSubmit={submitCorrection}><label className="field"><span>Data e hora</span><input autoFocus required type="datetime-local" value={editForm.inspectedAt} onChange={(e) => setEditForm({ ...editForm, inspectedAt: e.target.value })} /></label><label className="field"><span>Força</span><select value={editForm.strength} onChange={(e) => setEditForm({ ...editForm, strength: e.target.value })}>{strengthOptions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label><label className="field"><span>Rainha presente</span><select value={editQueen} onChange={(e) => setEditQueen(e.target.value)}><option value="">Não avaliado</option><option value="yes">Sim</option><option value="no">Não</option></select></label><label className="field"><span>Postura</span><input value={editForm.layingStatus} onChange={(e) => setEditForm({ ...editForm, layingStatus: e.target.value })} /></label><label className="field"><span>Reservas</span><input value={editForm.foodReserves} onChange={(e) => setEditForm({ ...editForm, foodReserves: e.target.value })} /></label><label className="field"><span>Crias</span><input value={editForm.broodStatus} onChange={(e) => setEditForm({ ...editForm, broodStatus: e.target.value })} /></label><label className="field full"><span>Pragas</span><input value={editForm.pestsNotes} onChange={(e) => setEditForm({ ...editForm, pestsNotes: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={2} value={editForm.observations} onChange={(e) => setEditForm({ ...editForm, observations: e.target.value })} /></label><label className="field full"><span>Ações realizadas</span><textarea rows={2} value={editForm.actionsTaken} onChange={(e) => setEditForm({ ...editForm, actionsTaken: e.target.value })} /></label><label className="field"><span>Próxima inspeção</span><input type="datetime-local" value={editForm.nextInspectionAt} onChange={(e) => setEditForm({ ...editForm, nextInspectionAt: e.target.value })} /></label><label className="field full"><span>Motivo da correção</span><textarea required rows={3} value={editForm.reason} onChange={(e) => setEditForm({ ...editForm, reason: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setEditForm(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !editForm.reason.trim()}>Salvar correção</button></div></form>}</Dialog>
    <ReasonDialog open={Boolean(voidTarget)} title="Anular inspeção" description={voidTarget ? `${voidTarget.colonyCode} · ${formatDateTimeBr(voidTarget.inspectedAt)}` : ""} confirmLabel="Anular registro" consequence="O registro continuará no histórico, mas deixará de determinar força atual, alertas e próxima inspeção válida." danger busy={busy} onClose={() => setVoidTarget(null)} onConfirm={async (reason) => { if (!voidTarget) return false; const ok = await onVoid({ id: voidTarget.id, reason }); if (ok) await reload(); return ok; }} />
  </div>;
}

function InspectionForm({ form, setForm, queenValue, setQueenValue, colonies, busy, onSubmit, submitLabel }: { form: CreateInspectionInput; setForm: (value: CreateInspectionInput) => void; queenValue: string; setQueenValue: (value: string) => void; colonies: Colony[]; busy: boolean; onSubmit: (event: FormEvent) => void; submitLabel: string }) { return <form className="form-grid" onSubmit={onSubmit}><label className="field full"><span>Colônia</span><select autoFocus required value={form.colonyId} onChange={(e) => setForm({ ...form, colonyId: e.target.value })}><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code}</option>)}</select></label><label className="field"><span>Data e hora</span><input type="datetime-local" value={form.inspectedAt} onChange={(e) => setForm({ ...form, inspectedAt: e.target.value })} /></label><label className="field"><span>Força</span><select value={form.strength} onChange={(e) => setForm({ ...form, strength: e.target.value })}>{strengthOptions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label><label className="field"><span>Rainha presente</span><select value={queenValue} onChange={(e) => setQueenValue(e.target.value)}><option value="">Não avaliado</option><option value="yes">Sim</option><option value="no">Não</option></select></label><label className="field"><span>Postura</span><input value={form.layingStatus} onChange={(e) => setForm({ ...form, layingStatus: e.target.value })} /></label><label className="field"><span>Reservas</span><input value={form.foodReserves} onChange={(e) => setForm({ ...form, foodReserves: e.target.value })} /></label><label className="field"><span>Crias</span><input value={form.broodStatus} onChange={(e) => setForm({ ...form, broodStatus: e.target.value })} /></label><label className="field full"><span>Pragas</span><input value={form.pestsNotes} onChange={(e) => setForm({ ...form, pestsNotes: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={2} value={form.observations} onChange={(e) => setForm({ ...form, observations: e.target.value })} /></label><label className="field full"><span>Ações realizadas</span><textarea rows={2} value={form.actionsTaken} onChange={(e) => setForm({ ...form, actionsTaken: e.target.value })} /></label><label className="field full"><span>Próxima inspeção</span><input type="datetime-local" value={form.nextInspectionAt} onChange={(e) => setForm({ ...form, nextInspectionAt: e.target.value })} /></label><div className="form-actions full"><button type="submit" disabled={busy || !form.colonyId}>{submitLabel}</button></div></form>; }
function normalizeDateTime(value?: string) { if (!value) return undefined; const normalized = value.replace("T", " "); return normalized.length === 16 ? `${normalized}:00` : normalized; }
function toInputDateTime(value: string) { return value.replace(" ", "T").slice(0, 16); }
function strengthLabel(value: string) { return strengthOptions.find(([key]) => key === value)?.[1] || value; }
function booleanLabel(value?: boolean | null) { if (value === true) return "Presente"; if (value === false) return "Não observada"; return "Não avaliado"; }
