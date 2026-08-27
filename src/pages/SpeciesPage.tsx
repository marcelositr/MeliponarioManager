import { useMemo, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import type { CreateSpeciesInput, Species } from "../types";

type SpeciesPageProps = { items: Species[]; busy: boolean; onCreate: (input: CreateSpeciesInput) => Promise<boolean>; };
const initialForm: CreateSpeciesInput = { commonName: "", scientificName: "", genus: "", notes: "" };

export function SpeciesPage({ items, busy, onCreate }: SpeciesPageProps) {
  const [form, setForm] = useState<CreateSpeciesInput>(initialForm);
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const filtered = useMemo(() => { const query = search.trim().toLocaleLowerCase(); return query ? items.filter((item) => [item.commonName, item.scientificName, item.genus].some((value) => value?.toLocaleLowerCase().includes(query))) : items; }, [items, search]);
  async function submit(event: FormEvent) { event.preventDefault(); if (await onCreate(form)) { setForm(initialForm); setOpen(false); } }
  function close() { if (!busy) { setOpen(false); setForm(initialForm); } }

  return <div className="page-stack">
    <PageToolbar title="Espécies" description="Catálogo técnico compartilhado por todo o plantel." count={`${items.length} cadastradas`} search={{ value: search, onChange: setSearch, placeholder: "Buscar espécie..." }} primaryAction={{ label: "Nova espécie", onClick: () => setOpen(true), disabled: busy }} />
    <section className="panel wide-list"><div className="panel-heading"><h2>Catálogo atual</h2><p>Uma referência única evita nomes divergentes no histórico.</p></div>{filtered.length === 0 ? <div className="empty-list">{items.length === 0 ? "Nenhuma espécie cadastrada ainda." : "Nenhum resultado para a busca."}</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Nome popular</th><th>Nome científico</th><th>Gênero</th><th>Observações</th></tr></thead><tbody>{filtered.map((item) => <tr key={item.id}><td><strong>{item.commonName}</strong></td><td className="scientific-name">{item.scientificName || "Não informado"}</td><td>{item.genus || "Não informado"}</td><td>{item.notes || "—"}</td></tr>)}</tbody></table></div>}</section>
    <Dialog open={open} onClose={close} title="Nova espécie" description="Use o nome popular no dia a dia e complete os dados técnicos quando souber." size="medium"><form className="form-grid" onSubmit={submit}><label className="field full"><span>Nome popular</span><input autoFocus required value={form.commonName} onChange={(e) => setForm({ ...form, commonName: e.target.value })} placeholder="Ex.: Jataí" /></label><label className="field"><span>Nome científico</span><input value={form.scientificName} onChange={(e) => setForm({ ...form, scientificName: e.target.value })} /></label><label className="field"><span>Gênero</span><input value={form.genus} onChange={(e) => setForm({ ...form, genus: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={4} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={close} disabled={busy}>Cancelar</button><button type="submit" disabled={busy}>{busy ? "Salvando..." : "Salvar espécie"}</button></div></form></Dialog>
  </div>;
}
