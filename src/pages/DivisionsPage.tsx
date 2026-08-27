import { useEffect, useMemo, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { getColonyGenealogy, listColonyDivisions } from "../lib/api";
import type { Colony, ColonyDivision, CreateDivisionInput, GenealogyNode } from "../types";

type DivisionsPageProps = { colonies: Colony[]; busy: boolean; onCreate: (input: CreateDivisionInput) => Promise<boolean>; };
const initialForm: CreateDivisionInput = { parentColonyId: "", daughterCode: "", daughterNotes: "", performedAt: "", result: "successful", notes: "" };

export function DivisionsPage({ colonies, busy, onCreate }: DivisionsPageProps) {
  const [selectedColonyId, setSelectedColonyId] = useState("");
  const [form, setForm] = useState<CreateDivisionInput>(initialForm);
  const [divisions, setDivisions] = useState<ColonyDivision[]>([]);
  const [genealogy, setGenealogy] = useState<GenealogyNode[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadError, setLoadError] = useState("");
  const [open, setOpen] = useState(false);
  const activeColonies = useMemo(() => colonies.filter((colony) => !["lost", "inactive", "transferred"].includes(colony.status)), [colonies]);

  useEffect(() => {
    let cancelled = false;
    if (!selectedColonyId) { setDivisions([]); setGenealogy([]); setLoadError(""); return; }
    setLoading(true); setLoadError("");
    Promise.all([listColonyDivisions(selectedColonyId), getColonyGenealogy(selectedColonyId)]).then(([divisionItems, tree]) => { if (!cancelled) { setDivisions(divisionItems); setGenealogy(tree); } }).catch(() => { if (!cancelled) setLoadError("Não foi possível carregar divisões e genealogia desta colônia."); }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [selectedColonyId]);

  async function reload(colonyId = selectedColonyId) { if (!colonyId) return; const [divisionItems, tree] = await Promise.all([listColonyDivisions(colonyId), getColonyGenealogy(colonyId)]); setDivisions(divisionItems); setGenealogy(tree); }
  function openNew() { setForm({ ...initialForm, parentColonyId: selectedColonyId }); setOpen(true); }
  async function submit(event: FormEvent) { event.preventDefault(); const result = form.result || "successful"; const input: CreateDivisionInput = { ...form, performedAt: normalizeDateTime(form.performedAt), daughterCode: result === "failed" ? undefined : form.daughterCode, daughterNotes: result === "failed" ? undefined : form.daughterNotes }; if (await onCreate(input)) { setSelectedColonyId(form.parentColonyId); setOpen(false); setForm(initialForm); await reload(form.parentColonyId); } }

  return <div className="page-stack">
    <PageToolbar title="Divisões e genealogia" description="Multiplicações preservando identidade e origem das colônias." count={selectedColonyId ? `${genealogy.length} na árvore` : `${activeColonies.length} manejáveis`} primaryAction={{ label: "Nova divisão", onClick: openNew, disabled: busy || activeColonies.length === 0 }}><label className="toolbar-select"><span className="sr-only">Colônia</span><select value={selectedColonyId} onChange={(e) => setSelectedColonyId(e.target.value)}><option value="">Selecione uma colônia...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code}</option>)}</select></label></PageToolbar>
    {loadError && <div className="inline-notice">{loadError}</div>}
    <div className="content-grid">
      <section className="panel wide-list"><div className="panel-heading"><h2>Genealogia</h2><p>Geração 0 é a colônia selecionada; descendentes aparecem nas gerações seguintes.</p></div>{!selectedColonyId ? <div className="empty-list">Selecione uma colônia na toolbar.</div> : loading ? <div className="empty-list">Carregando genealogia...</div> : genealogy.length === 0 ? <div className="empty-list">Nenhum dado genealógico.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Colônia</th><th>Colônia-mãe</th><th>Geração</th></tr></thead><tbody>{genealogy.map((node) => <tr key={node.colonyId}><td><strong>{node.code}</strong></td><td>{node.motherColonyCode || "Raiz selecionada"}</td><td>{node.generation}</td></tr>)}</tbody></table></div>}</section>
      <section className="panel wide-list"><div className="panel-heading"><h2>Histórico de divisões</h2><p>Participações da colônia selecionada como mãe ou filha.</p></div>{!selectedColonyId ? <div className="empty-list">Selecione uma colônia.</div> : divisions.length === 0 ? <div className="empty-list">Nenhuma divisão registrada.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Data</th><th>Mãe</th><th>Filha</th><th>Caixa</th><th>Resultado</th></tr></thead><tbody>{divisions.map((item) => <tr key={item.id}><td><strong>{formatDateTime(item.performedAt)}</strong></td><td>{item.parentColonyCode}</td><td>{item.daughterColonyCode || "Sem filha"}</td><td>{item.sourceBoxCode || "—"}</td><td>{resultLabel(item.result)}</td></tr>)}</tbody></table></div>}</section>
    </div>
    <Dialog open={open} onClose={() => !busy && setOpen(false)} title="Nova divisão" description="Resultados bem-sucedidos ou parciais criam a colônia filha na mesma espécie e meliponário da mãe." size="medium"><form className="form-grid" onSubmit={submit}><label className="field full"><span>Colônia mãe</span><select autoFocus required value={form.parentColonyId} onChange={(e) => setForm({ ...form, parentColonyId: e.target.value })}><option value="">Selecione...</option>{activeColonies.map((c) => <option value={c.id} key={c.id}>{c.code} {c.currentBoxCode ? `· ${c.currentBoxCode}` : "· sem caixa"}</option>)}</select></label><label className="field"><span>Resultado</span><select value={form.result} onChange={(e) => setForm({ ...form, result: e.target.value })}><option value="successful">Bem-sucedida</option><option value="partial">Parcial</option><option value="failed">Não vingou</option></select></label><label className="field"><span>Data e hora</span><input type="datetime-local" value={form.performedAt} onChange={(e) => setForm({ ...form, performedAt: e.target.value })} /></label>{form.result !== "failed" && <><label className="field full"><span>Código da colônia filha</span><input required value={form.daughterCode} onChange={(e) => setForm({ ...form, daughterCode: e.target.value })} /></label><label className="field full"><span>Notas da filha</span><textarea rows={2} value={form.daughterNotes} onChange={(e) => setForm({ ...form, daughterNotes: e.target.value })} /></label></>}<label className="field full"><span>Observações da divisão</span><textarea rows={3} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setOpen(false)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !form.parentColonyId}>{busy ? "Salvando..." : "Registrar divisão"}</button></div></form></Dialog>
  </div>;
}
function normalizeDateTime(value?: string) { if (!value) return undefined; const n = value.replace("T", " "); return n.length === 16 ? `${n}:00` : n; }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function resultLabel(value: string) { return value === "successful" ? "Bem-sucedida" : value === "partial" ? "Parcial" : "Não vingou"; }
