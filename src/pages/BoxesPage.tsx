import { useMemo, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import type { CreateBoxInput, HiveBox, Meliponary } from "../types";

type BoxesPageProps = { items: HiveBox[]; meliponaries: Meliponary[]; busy: boolean; onCreate: (input: CreateBoxInput) => Promise<boolean>; };
const initialForm: CreateBoxInput = { meliponaryId: "", code: "", model: "", material: "", locationNote: "", notes: "" };

export function BoxesPage({ items, meliponaries, busy, onCreate }: BoxesPageProps) {
  const [form, setForm] = useState<CreateBoxInput>(initialForm);
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [status, setStatus] = useState("all");
  const meliponaryNames = useMemo(() => new Map(meliponaries.map((item) => [item.id, item.name])), [meliponaries]);
  const filtered = useMemo(() => { const query = search.trim().toLocaleLowerCase(); return items.filter((item) => (status === "all" || item.status === status) && (!query || [item.code, item.model, item.material, item.locationNote, item.currentColonyCode, meliponaryNames.get(item.meliponaryId)].some((value) => value?.toLocaleLowerCase().includes(query)))); }, [items, meliponaryNames, search, status]);
  async function submit(event: FormEvent) { event.preventDefault(); if (await onCreate(form)) { setForm({ ...initialForm, meliponaryId: form.meliponaryId }); setOpen(false); } }
  function close() { if (!busy) setOpen(false); }

  return <div className="page-stack">
    <PageToolbar title="Caixas" description="Estrutura física separada da identidade das colônias." count={`${items.length} cadastradas`} search={{ value: search, onChange: setSearch, placeholder: "Buscar caixa..." }} primaryAction={{ label: "Nova caixa", onClick: () => setOpen(true), disabled: busy || meliponaries.length === 0 }}><label className="toolbar-select"><span className="sr-only">Estado</span><select value={status} onChange={(e) => setStatus(e.target.value)}><option value="all">Todos os estados</option><option value="active">Ativas</option><option value="maintenance">Manutenção</option><option value="retired">Aposentadas</option></select></label></PageToolbar>
    {meliponaries.length === 0 && <div className="inline-notice">Cadastre um meliponário antes de adicionar caixas.</div>}
    <section className="panel wide-list"><div className="panel-heading"><h2>Caixas cadastradas</h2><p>Ocupação atual e estado físico, sem apagar o histórico anterior.</p></div>{filtered.length === 0 ? <div className="empty-list">{items.length === 0 ? "Nenhuma caixa cadastrada ainda." : "Nenhuma caixa corresponde aos filtros."}</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Código</th><th>Meliponário</th><th>Colônia atual</th><th>Estado</th><th>Modelo</th><th>Material</th><th>Posição</th></tr></thead><tbody>{filtered.map((item) => <tr key={item.id}><td><strong>{item.code}</strong></td><td>{meliponaryNames.get(item.meliponaryId) || "Meliponário"}</td><td>{item.currentColonyCode || "Livre"}</td><td><span className={`badge status-${item.status}`}>{boxStatusLabel(item.status)}</span></td><td>{item.model || "—"}</td><td>{item.material || "—"}</td><td>{item.locationNote || "—"}</td></tr>)}</tbody></table></div>}</section>
    <Dialog open={open} onClose={close} title="Nova caixa" description="O código precisa ser único dentro do meliponário." size="medium"><form className="form-grid" onSubmit={submit}><label className="field full"><span>Meliponário</span><select autoFocus required value={form.meliponaryId} onChange={(e) => setForm({ ...form, meliponaryId: e.target.value })}><option value="">Selecione...</option>{meliponaries.map((item) => <option value={item.id} key={item.id}>{item.name}</option>)}</select></label><label className="field"><span>Código</span><input required value={form.code} onChange={(e) => setForm({ ...form, code: e.target.value })} placeholder="Ex.: CX-001" /></label><label className="field"><span>Modelo</span><input value={form.model} onChange={(e) => setForm({ ...form, model: e.target.value })} /></label><label className="field"><span>Material</span><input value={form.material} onChange={(e) => setForm({ ...form, material: e.target.value })} /></label><label className="field"><span>Posição / local</span><input value={form.locationNote} onChange={(e) => setForm({ ...form, locationNote: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={3} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={close} disabled={busy}>Cancelar</button><button type="submit" disabled={busy}>{busy ? "Salvando..." : "Salvar caixa"}</button></div></form></Dialog>
  </div>;
}

function boxStatusLabel(status: string) { const labels: Record<string, string> = { active: "Ativa", maintenance: "Manutenção", retired: "Aposentada" }; return labels[status] || status; }
