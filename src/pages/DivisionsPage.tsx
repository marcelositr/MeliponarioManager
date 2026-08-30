import { useEffect, useMemo, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { RecordActions } from "../components/RecordActions";
import { RecordStateBadge } from "../components/RecordStateBadge";
import type { RecordStateMap } from "../hooks/useAppData";
import { getColonyGenealogy, listColonyDivisions } from "../lib/api";
import type { Colony, ColonyDivision, CorrectDivisionInput, CreateDivisionInput, GenealogyNode, VoidDivisionInput } from "../types";

type Props = {
  colonies: Colony[];
  busy: boolean;
  recordStateMap: RecordStateMap;
  onCreate: (input: CreateDivisionInput) => Promise<boolean>;
  onCorrect: (input: CorrectDivisionInput) => Promise<boolean>;
  onVoid: (input: VoidDivisionInput) => Promise<boolean>;
};
const initialForm: CreateDivisionInput = { parentColonyId: "", daughterCode: "", daughterNotes: "", performedAt: "", result: "successful", notes: "" };

export function DivisionsPage({ colonies, busy, recordStateMap, onCreate, onCorrect, onVoid }: Props) {
  const [selectedColonyId, setSelectedColonyId] = useState("");
  const [form, setForm] = useState<CreateDivisionInput>(initialForm);
  const [divisions, setDivisions] = useState<ColonyDivision[]>([]);
  const [genealogy, setGenealogy] = useState<GenealogyNode[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadError, setLoadError] = useState("");
  const [open, setOpen] = useState(false);
  const [detail, setDetail] = useState<ColonyDivision | null>(null);
  const [editForm, setEditForm] = useState<CorrectDivisionInput | null>(null);
  const [voidTarget, setVoidTarget] = useState<ColonyDivision | null>(null);
  const [voidReason, setVoidReason] = useState("");
  const [daughterDisposition, setDaughterDisposition] = useState<"keep" | "deactivate">("keep");
  const activeColonies = useMemo(() => colonies.filter((colony) => !["lost", "inactive", "transferred"].includes(colony.status)), [colonies]);

  useEffect(() => { void reload(selectedColonyId); }, [selectedColonyId]);
  async function reload(colonyId = selectedColonyId) {
    if (!colonyId) { setDivisions([]); setGenealogy([]); setLoadError(""); return; }
    setLoading(true); setLoadError("");
    try { const [divisionItems, tree] = await Promise.all([listColonyDivisions(colonyId), getColonyGenealogy(colonyId)]); setDivisions(divisionItems); setGenealogy(tree); }
    catch { setLoadError("Não foi possível carregar divisões e genealogia desta colônia."); }
    finally { setLoading(false); }
  }
  function openNew() { setForm({ ...initialForm, parentColonyId: selectedColonyId }); setOpen(true); }
  async function submit(event: FormEvent) { event.preventDefault(); const result = form.result || "successful"; const input: CreateDivisionInput = { ...form, performedAt: normalizeDateTime(form.performedAt), daughterCode: result === "failed" ? undefined : form.daughterCode, daughterNotes: result === "failed" ? undefined : form.daughterNotes }; if (await onCreate(input)) { setSelectedColonyId(form.parentColonyId); setOpen(false); setForm(initialForm); await reload(form.parentColonyId); } }
  function beginCorrect(item: ColonyDivision) { setEditForm({ id: item.id, reason: "", notes: item.notes || "" }); }
  async function submitCorrection(event: FormEvent) { event.preventDefault(); if (!editForm) return; if (await onCorrect(editForm)) { setEditForm(null); await reload(); } }
  function beginVoid(item: ColonyDivision) { setVoidTarget(item); setVoidReason(""); setDaughterDisposition("keep"); }
  async function submitVoid(event: FormEvent) { event.preventDefault(); if (!voidTarget) return; const input: VoidDivisionInput = { id: voidTarget.id, reason: voidReason, daughterDisposition: voidTarget.daughterColonyId ? daughterDisposition : undefined }; if (await onVoid(input)) { setVoidTarget(null); setVoidReason(""); await reload(); } }

  return <div className="page-stack">
    <PageToolbar title="Divisões e genealogia" description="Multiplicações preservando identidade e origem das colônias." count={selectedColonyId ? `${genealogy.length} na árvore` : `${activeColonies.length} manejáveis`} primaryAction={{ label: "Nova divisão", onClick: openNew, disabled: busy || activeColonies.length === 0 }}><label className="toolbar-select"><span className="sr-only">Colônia</span><select value={selectedColonyId} onChange={(e) => setSelectedColonyId(e.target.value)}><option value="">Selecione uma colônia...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code}</option>)}</select></label></PageToolbar>
    {loadError && <div className="inline-notice">{loadError}</div>}
    <div className="content-grid">
      <section className="panel wide-list"><div className="panel-heading"><h2>Genealogia</h2><p>Geração 0 é a colônia selecionada; descendentes aparecem nas gerações seguintes.</p></div>{!selectedColonyId ? <div className="empty-list">Selecione uma colônia na toolbar.</div> : loading ? <div className="empty-list">Carregando genealogia...</div> : genealogy.length === 0 ? <div className="empty-list">Nenhum dado genealógico.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Colônia</th><th>Colônia-mãe</th><th>Geração</th></tr></thead><tbody>{genealogy.map((node) => <tr key={node.colonyId}><td><strong>{node.code}</strong></td><td>{node.motherColonyCode || "Raiz selecionada"}</td><td>{node.generation}</td></tr>)}</tbody></table></div>}</section>
      <section className="panel wide-list"><div className="panel-heading"><h2>Histórico de divisões</h2><p>Anulação de divisão com filha exige decisão explícita e é bloqueada se já existirem consequências posteriores.</p></div>{!selectedColonyId ? <div className="empty-list">Selecione uma colônia.</div> : divisions.length === 0 ? <div className="empty-list">Nenhuma divisão registrada.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Data</th><th>Mãe</th><th>Filha</th><th>Resultado</th><th>Estado</th><th>Ações</th></tr></thead><tbody>{divisions.map((item) => { const state = recordStateMap.get(`division:${item.id}`); return <tr key={item.id} className={state?.voidedAt ? "voided-row" : undefined}><td><strong>{formatDateTime(item.performedAt)}</strong></td><td>{item.parentColonyCode}</td><td>{item.daughterColonyCode || "Sem filha"}</td><td>{resultLabel(item.result)}</td><td><RecordStateBadge state={state} /></td><td><RecordActions busy={busy} onOpen={() => setDetail(item)} onEdit={state?.voidedAt ? undefined : () => beginCorrect(item)} secondary={[{ label: "Anular", onClick: () => beginVoid(item), disabled: Boolean(state?.voidedAt), danger: true }]} /></td></tr>; })}</tbody></table></div>}</section>
    </div>

    <Dialog open={open} onClose={() => !busy && setOpen(false)} title="Nova divisão" description="Resultados bem-sucedidos ou parciais criam a colônia filha na mesma espécie e meliponário da mãe." size="medium"><form className="form-grid" onSubmit={submit}><label className="field full"><span>Colônia mãe</span><select autoFocus required value={form.parentColonyId} onChange={(e) => setForm({ ...form, parentColonyId: e.target.value })}><option value="">Selecione...</option>{activeColonies.map((c) => <option value={c.id} key={c.id}>{c.code} {c.currentBoxCode ? `· ${c.currentBoxCode}` : "· sem caixa"}</option>)}</select></label><label className="field"><span>Resultado</span><select value={form.result} onChange={(e) => setForm({ ...form, result: e.target.value })}><option value="successful">Bem-sucedida</option><option value="partial">Parcial</option><option value="failed">Não vingou</option></select></label><label className="field"><span>Data e hora</span><input type="datetime-local" value={form.performedAt} onChange={(e) => setForm({ ...form, performedAt: e.target.value })} /></label>{form.result !== "failed" && <><label className="field full"><span>Código da colônia filha</span><input required value={form.daughterCode} onChange={(e) => setForm({ ...form, daughterCode: e.target.value })} /></label><label className="field full"><span>Notas da filha</span><textarea rows={2} value={form.daughterNotes} onChange={(e) => setForm({ ...form, daughterNotes: e.target.value })} /></label></>}<label className="field full"><span>Observações da divisão</span><textarea rows={3} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setOpen(false)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !form.parentColonyId}>{busy ? "Salvando..." : "Registrar divisão"}</button></div></form></Dialog>
    <Dialog open={Boolean(detail)} onClose={() => setDetail(null)} title="Divisão" description={detail ? `${detail.parentColonyCode} · ${formatDateTime(detail.performedAt)}` : ""} size="medium">{detail && <div className="detail-grid"><div><span>Mãe</span><strong>{detail.parentColonyCode}</strong></div><div><span>Filha</span><strong>{detail.daughterColonyCode || "Sem filha"}</strong></div><div><span>Caixa de origem</span><strong>{detail.sourceBoxCode || "—"}</strong></div><div><span>Resultado</span><strong>{resultLabel(detail.result)}</strong></div><div className="full"><span>Observações</span><p>{detail.notes || "—"}</p></div></div>}</Dialog>
    <Dialog open={Boolean(editForm)} onClose={() => !busy && setEditForm(null)} title="Corrigir observações da divisão" description="Relações de mãe/filha, data e resultado são consequenciais e não são reescritos por esta correção." size="medium">{editForm && <form className="form-grid" onSubmit={submitCorrection}><label className="field full"><span>Observações</span><textarea autoFocus rows={4} value={editForm.notes || ""} onChange={(e) => setEditForm({ ...editForm, notes: e.target.value })} /></label><label className="field full"><span>Motivo da correção</span><textarea required rows={3} value={editForm.reason} onChange={(e) => setEditForm({ ...editForm, reason: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setEditForm(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !editForm.reason.trim()}>Salvar correção</button></div></form>}</Dialog>
    <Dialog open={Boolean(voidTarget)} onClose={() => !busy && setVoidTarget(null)} title="Anular divisão" description="A divisão continuará na auditoria; o backend bloqueia automaticamente qualquer anulação que reescreveria consequências já existentes." size="medium">{voidTarget && <form className="form-grid" onSubmit={submitVoid}>{voidTarget.daughterColonyId && <label className="field full"><span>Destino explícito da colônia filha</span><select value={daughterDisposition} onChange={(e) => setDaughterDisposition(e.target.value as "keep" | "deactivate")}><option value="keep">Preservar a filha como está</option><option value="deactivate">Inativar a filha</option></select></label>}<label className="field full"><span>Motivo da anulação</span><textarea autoFocus required rows={4} value={voidReason} onChange={(e) => setVoidReason(e.target.value)} /></label><div className="inline-notice field full">Se a filha já tiver ocupação, manejo, produção, movimentação, descendência ou qualquer consequência histórica, a anulação será recusada sem alterar dados.</div><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setVoidTarget(null)} disabled={busy}>Cancelar</button><button className="button-danger" type="submit" disabled={busy || !voidReason.trim()}>Anular divisão</button></div></form>}</Dialog>
  </div>;
}
function normalizeDateTime(value?: string) { if (!value) return undefined; const n = value.replace("T", " "); return n.length === 16 ? `${n}:00` : n; }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function resultLabel(value: string) { return value === "successful" ? "Bem-sucedida" : value === "partial" ? "Parcial" : "Não vingou"; }
