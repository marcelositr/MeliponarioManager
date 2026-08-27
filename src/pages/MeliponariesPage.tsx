import { useMemo, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import type { CreateMeliponaryInput, Meliponary } from "../types";

type MeliponariesPageProps = { items: Meliponary[]; busy: boolean; onCreate: (input: CreateMeliponaryInput) => Promise<boolean>; };
const initialForm: CreateMeliponaryInput = { name: "", responsibleName: "", location: "", notes: "" };

export function MeliponariesPage({ items, busy, onCreate }: MeliponariesPageProps) {
  const [form, setForm] = useState<CreateMeliponaryInput>(initialForm);
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const filtered = useMemo(() => { const query = search.trim().toLocaleLowerCase(); return query ? items.filter((item) => [item.name, item.location, item.responsibleName].some((value) => value?.toLocaleLowerCase().includes(query))) : items; }, [items, search]);

  async function submit(event: FormEvent) { event.preventDefault(); if (await onCreate(form)) { setForm(initialForm); setOpen(false); } }
  function close() { if (!busy) { setOpen(false); setForm(initialForm); } }

  return <div className="page-stack">
    <PageToolbar title="Meliponários" description="Unidades físicas de criação e contexto operacional." count={`${items.length} cadastrados`} search={{ value: search, onChange: setSearch, placeholder: "Buscar meliponário..." }} primaryAction={{ label: "Novo meliponário", onClick: () => setOpen(true), disabled: busy }} />
    <section className="panel wide-list"><div className="panel-heading"><h2>Locais cadastrados</h2><p>A base física do plantel. A seleção global no topo não filtra silenciosamente esta lista nesta etapa.</p></div>{filtered.length === 0 ? <div className="empty-list">{items.length === 0 ? "Nenhum meliponário cadastrado ainda." : "Nenhum resultado para a busca."}</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Nome</th><th>Localização</th><th>Responsável</th><th>Observações</th></tr></thead><tbody>{filtered.map((item) => <tr key={item.id}><td><strong>{item.name}</strong></td><td>{item.location || "Não informada"}</td><td>{item.responsibleName || "Não informado"}</td><td>{item.notes || "—"}</td></tr>)}</tbody></table></div>}</section>
    <Dialog open={open} onClose={close} title="Novo meliponário" description="Somente o nome é obrigatório." size="medium"><form className="form-grid" onSubmit={submit}><label className="field full"><span>Nome</span><input autoFocus required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} placeholder="Ex.: Meliponário principal" /></label><label className="field"><span>Responsável</span><input value={form.responsibleName} onChange={(e) => setForm({ ...form, responsibleName: e.target.value })} /></label><label className="field"><span>Localização</span><input value={form.location} onChange={(e) => setForm({ ...form, location: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={4} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={close} disabled={busy}>Cancelar</button><button type="submit" disabled={busy}>{busy ? "Salvando..." : "Salvar meliponário"}</button></div></form></Dialog>
  </div>;
}
