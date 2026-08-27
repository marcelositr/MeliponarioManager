import { useEffect, useMemo, useState, type FormEvent } from "react";
import { getColonyGenealogy, listColonyDivisions } from "../lib/api";
import type { Colony, ColonyDivision, CreateDivisionInput, GenealogyNode } from "../types";

type DivisionsPageProps = { colonies: Colony[]; busy: boolean; onCreate: (input: CreateDivisionInput) => Promise<boolean>; };
const initialForm: CreateDivisionInput = { parentColonyId: "", daughterCode: "", daughterNotes: "", performedAt: "", result: "successful", notes: "" };

export function DivisionsPage({ colonies, busy, onCreate }: DivisionsPageProps) {
  const [form, setForm] = useState<CreateDivisionInput>(initialForm);
  const [divisions, setDivisions] = useState<ColonyDivision[]>([]);
  const [genealogy, setGenealogy] = useState<GenealogyNode[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadError, setLoadError] = useState("");
  const activeColonies = useMemo(() => colonies.filter((colony) => !["lost", "inactive", "transferred"].includes(colony.status)), [colonies]);

  useEffect(() => {
    let cancelled = false;
    if (!form.parentColonyId) { setDivisions([]); setGenealogy([]); setLoadError(""); return; }
    setLoading(true); setLoadError("");
    Promise.all([listColonyDivisions(form.parentColonyId), getColonyGenealogy(form.parentColonyId)])
      .then(([divisionItems, tree]) => { if (!cancelled) { setDivisions(divisionItems); setGenealogy(tree); } })
      .catch(() => { if (!cancelled) setLoadError("Não foi possível carregar divisões e genealogia desta colônia."); })
      .finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [form.parentColonyId]);

  async function reload(colonyId = form.parentColonyId) {
    if (!colonyId) return;
    const [divisionItems, tree] = await Promise.all([listColonyDivisions(colonyId), getColonyGenealogy(colonyId)]);
    setDivisions(divisionItems); setGenealogy(tree);
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    const result = form.result || "successful";
    const input: CreateDivisionInput = { ...form, performedAt: normalizeDateTime(form.performedAt), daughterCode: result === "failed" ? undefined : form.daughterCode, daughterNotes: result === "failed" ? undefined : form.daughterNotes };
    if (await onCreate(input)) {
      const parentColonyId = form.parentColonyId;
      setForm({ ...initialForm, parentColonyId });
      await reload(parentColonyId);
    }
  }

  return <div className="page-stack">
    <section className="page-heading"><div><span className="eyebrow">Multiplicação</span><h1>Divisões e genealogia</h1><p>Registre multiplicações mantendo a identidade da colônia mãe e acompanhe as gerações criadas ao longo do tempo.</p></div><span className="count-pill">{genealogy.length} colônia{genealogy.length === 1 ? "" : "s"} na árvore</span></section>
    <div className="content-grid">
      <section className="panel form-panel"><div className="panel-heading"><h2>Nova divisão</h2><p>Resultados bem-sucedidos ou parciais criam a colônia filha na mesma espécie e meliponário da mãe.</p></div>
        {activeColonies.length === 0 ? <div className="inline-notice">Não há colônia ativa disponível para divisão.</div> : <form className="form-grid" onSubmit={submit}>
          <label className="field full"><span>Colônia mãe</span><select required value={form.parentColonyId} onChange={(e) => setForm({ ...form, parentColonyId: e.target.value })}><option value="">Selecione...</option>{activeColonies.map((c) => <option value={c.id} key={c.id}>{c.code} {c.currentBoxCode ? `· ${c.currentBoxCode}` : "· sem caixa"}</option>)}</select></label>
          <label className="field"><span>Resultado</span><select value={form.result} onChange={(e) => setForm({ ...form, result: e.target.value })}><option value="successful">Bem-sucedida</option><option value="partial">Parcial</option><option value="failed">Não vingou</option></select></label>
          <label className="field"><span>Data e hora</span><input type="datetime-local" value={form.performedAt} onChange={(e) => setForm({ ...form, performedAt: e.target.value })} /></label>
          {form.result !== "failed" && <><label className="field full"><span>Código da colônia filha</span><input required value={form.daughterCode} onChange={(e) => setForm({ ...form, daughterCode: e.target.value })} placeholder="Ex.: JAT-004" /></label><label className="field full"><span>Notas da filha</span><textarea rows={2} value={form.daughterNotes} onChange={(e) => setForm({ ...form, daughterNotes: e.target.value })} /></label></>}
          <label className="field full"><span>Observações da divisão</span><textarea rows={3} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} /></label>
          <div className="form-actions full"><button disabled={busy || !form.parentColonyId} type="submit">{busy ? "Salvando..." : "Registrar divisão"}</button></div>
        </form>}
      </section>
      <section className="panel list-panel"><div className="panel-heading"><h2>Genealogia</h2><p>A geração 0 é a colônia selecionada; as seguintes são descendentes diretas e indiretas.</p></div>
        {!form.parentColonyId ? <div className="empty-list">Selecione uma colônia mãe para visualizar a árvore.</div> : loading ? <div className="empty-list">Carregando genealogia...</div> : loadError ? <div className="inline-notice">{loadError}</div> : genealogy.length === 0 ? <div className="empty-list">Nenhum dado genealógico.</div> : <div className="record-list">{genealogy.map((node) => <article className="record-card" key={node.colonyId}><div className="record-title-row"><div><strong>{"↳ ".repeat(Math.max(0, node.generation))}{node.code}</strong><span>{node.motherColonyCode ? `Filha de ${node.motherColonyCode}` : "Raiz selecionada"}</span></div><span className="badge">Geração {node.generation}</span></div></article>)}</div>}
      </section>
    </div>
    <section className="panel"><div className="panel-heading"><h2>Histórico de divisões</h2><p>Inclui divisões em que a colônia selecionada participou como mãe ou filha.</p></div>
      {!form.parentColonyId ? <div className="empty-list">Selecione uma colônia.</div> : divisions.length === 0 ? <div className="empty-list">Nenhuma divisão registrada.</div> : <div className="record-list">{divisions.map((item) => <article className="record-card" key={item.id}><div className="record-title-row"><div><strong>{item.parentColonyCode} → {item.daughterColonyCode || "sem filha"}</strong><span>{formatDateTime(item.performedAt)} · {item.sourceBoxCode ? `Caixa ${item.sourceBoxCode}` : "sem caixa na data"}</span></div><span className="badge">{resultLabel(item.result)}</span></div>{item.notes && <p>{item.notes}</p>}</article>)}</div>}
    </section>
  </div>;
}

function normalizeDateTime(value?: string) { if (!value) return undefined; const n = value.replace("T", " "); return n.length === 16 ? `${n}:00` : n; }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function resultLabel(value: string) { return value === "successful" ? "Bem-sucedida" : value === "partial" ? "Parcial" : "Não vingou"; }
