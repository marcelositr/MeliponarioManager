import { useEffect, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { listColonyInspections } from "../lib/api";
import type { Colony, CreateInspectionInput, Inspection } from "../types";

type InspectionsPageProps = { colonies: Colony[]; busy: boolean; onCreate: (input: CreateInspectionInput) => Promise<boolean>; };
const initialForm: CreateInspectionInput = { colonyId: "", inspectedAt: "", strength: "unknown", layingStatus: "", foodReserves: "", broodStatus: "", pestsNotes: "", observations: "", actionsTaken: "", nextInspectionAt: "" };
const strengthOptions = [["strong", "Forte"], ["medium", "Média"], ["weak", "Fraca"], ["unknown", "Não avaliada"]] as const;

export function InspectionsPage({ colonies, busy, onCreate }: InspectionsPageProps) {
  const [selectedColonyId, setSelectedColonyId] = useState("");
  const [form, setForm] = useState<CreateInspectionInput>(initialForm);
  const [queenValue, setQueenValue] = useState("");
  const [items, setItems] = useState<Inspection[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadError, setLoadError] = useState("");
  const [open, setOpen] = useState(false);

  useEffect(() => {
    let cancelled = false;
    if (!selectedColonyId) { setItems([]); setLoadError(""); return; }
    setLoading(true); setLoadError("");
    listColonyInspections(selectedColonyId).then((data) => { if (!cancelled) setItems(data); }).catch(() => { if (!cancelled) setLoadError("Não foi possível carregar as inspeções desta colônia."); }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [selectedColonyId]);

  async function reload(colonyId = selectedColonyId) { if (colonyId) setItems(await listColonyInspections(colonyId)); }
  function openNew() { setForm({ ...initialForm, colonyId: selectedColonyId }); setQueenValue(""); setOpen(true); }
  async function submit(event: FormEvent) {
    event.preventDefault();
    const input: CreateInspectionInput = { ...form, inspectedAt: normalizeDateTime(form.inspectedAt), nextInspectionAt: normalizeDateTime(form.nextInspectionAt), queenPresent: queenValue === "yes" ? true : queenValue === "no" ? false : null };
    if (await onCreate(input)) { setSelectedColonyId(form.colonyId); setOpen(false); setForm(initialForm); setQueenValue(""); await reload(form.colonyId); }
  }

  return <div className="page-stack">
    <PageToolbar title="Inspeções" description="Condição observada e histórico cronológico por colônia." count={selectedColonyId ? `${items.length} registros` : `${colonies.length} colônias`} primaryAction={{ label: "Nova inspeção", onClick: openNew, disabled: busy || colonies.length === 0 }}><label className="toolbar-select"><span className="sr-only">Colônia</span><select value={selectedColonyId} onChange={(e) => setSelectedColonyId(e.target.value)}><option value="">Selecione uma colônia...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>)}</select></label></PageToolbar>
    {colonies.length === 0 ? <div className="inline-notice">Cadastre uma colônia antes de registrar inspeções.</div> : <section className="panel wide-list"><div className="panel-heading"><h2>Histórico da colônia</h2><p>A caixa exibida em cada registro é resolvida conforme a data da inspeção.</p></div>{!selectedColonyId ? <div className="empty-list">Selecione uma colônia na toolbar para consultar o histórico.</div> : loading ? <div className="empty-list">Carregando inspeções...</div> : loadError ? <div className="inline-notice">{loadError}</div> : items.length === 0 ? <div className="empty-list">Nenhuma inspeção registrada para esta colônia.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Data</th><th>Caixa</th><th>Força</th><th>Rainha</th><th>Postura</th><th>Reservas</th><th>Próxima inspeção</th></tr></thead><tbody>{items.map((item) => <tr key={item.id}><td><strong>{formatDateTime(item.inspectedAt)}</strong></td><td>{item.boxCode || "Sem caixa"}</td><td><span className={`badge status-${item.strength}`}>{strengthLabel(item.strength)}</span></td><td>{booleanLabel(item.queenPresent)}</td><td>{item.layingStatus || "—"}</td><td>{item.foodReserves || "—"}</td><td>{item.nextInspectionAt ? formatDateTime(item.nextInspectionAt) : "Sem agendamento"}</td></tr>)}</tbody></table></div>}</section>}
    <Dialog open={open} onClose={() => !busy && setOpen(false)} title="Nova inspeção" description="Registre somente o que foi observado; a força alimenta os alertas derivados." size="large"><form className="form-grid" onSubmit={submit}><label className="field full"><span>Colônia</span><select autoFocus required value={form.colonyId} onChange={(e) => setForm({ ...form, colonyId: e.target.value })}><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>)}</select></label><label className="field"><span>Data e hora</span><input type="datetime-local" value={form.inspectedAt} onChange={(e) => setForm({ ...form, inspectedAt: e.target.value })} /></label><label className="field"><span>Força</span><select value={form.strength} onChange={(e) => setForm({ ...form, strength: e.target.value })}>{strengthOptions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label><label className="field"><span>Rainha presente</span><select value={queenValue} onChange={(e) => setQueenValue(e.target.value)}><option value="">Não avaliado</option><option value="yes">Sim</option><option value="no">Não</option></select></label><label className="field"><span>Postura</span><input value={form.layingStatus} onChange={(e) => setForm({ ...form, layingStatus: e.target.value })} /></label><label className="field"><span>Reservas de alimento</span><input value={form.foodReserves} onChange={(e) => setForm({ ...form, foodReserves: e.target.value })} /></label><label className="field"><span>Condição das crias</span><input value={form.broodStatus} onChange={(e) => setForm({ ...form, broodStatus: e.target.value })} /></label><label className="field full"><span>Pragas ou inimigos</span><input value={form.pestsNotes} onChange={(e) => setForm({ ...form, pestsNotes: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={3} value={form.observations} onChange={(e) => setForm({ ...form, observations: e.target.value })} /></label><label className="field full"><span>Ações realizadas</span><textarea rows={2} value={form.actionsTaken} onChange={(e) => setForm({ ...form, actionsTaken: e.target.value })} /></label><label className="field full"><span>Próxima inspeção</span><input type="datetime-local" value={form.nextInspectionAt} onChange={(e) => setForm({ ...form, nextInspectionAt: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setOpen(false)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !form.colonyId}>{busy ? "Salvando..." : "Registrar inspeção"}</button></div></form></Dialog>
  </div>;
}

function normalizeDateTime(value?: string) { if (!value) return undefined; const normalized = value.replace("T", " "); return normalized.length === 16 ? `${normalized}:00` : normalized; }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function strengthLabel(value: string) { return strengthOptions.find(([key]) => key === value)?.[1] || value; }
function booleanLabel(value?: boolean | null) { if (value === true) return "Presente"; if (value === false) return "Não observada"; return "Não avaliado"; }
