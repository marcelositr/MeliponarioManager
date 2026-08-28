import { useMemo, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { ReasonDialog } from "../components/ReasonDialog";
import { RecordActions } from "../components/RecordActions";
import type { CreateMeliponaryInput, EditMeliponaryInput, EntityActionInput, Meliponary } from "../types";

type Props = {
  items: Meliponary[];
  busy: boolean;
  onCreate: (input: CreateMeliponaryInput) => Promise<boolean>;
  onEdit: (input: EditMeliponaryInput) => Promise<boolean>;
  onArchive: (input: EntityActionInput) => Promise<boolean>;
  onReactivate: (input: EntityActionInput) => Promise<boolean>;
  onDelete: (input: EntityActionInput) => Promise<boolean>;
};
const initialForm: CreateMeliponaryInput = { name: "", responsibleName: "", location: "", notes: "" };

type ReasonAction = { kind: "archive" | "reactivate" | "delete"; item: Meliponary } | null;

export function MeliponariesPage({ items, busy, onCreate, onEdit, onArchive, onReactivate, onDelete }: Props) {
  const [form, setForm] = useState<CreateMeliponaryInput>(initialForm);
  const [createOpen, setCreateOpen] = useState(false);
  const [detail, setDetail] = useState<Meliponary | null>(null);
  const [editing, setEditing] = useState<Meliponary | null>(null);
  const [editForm, setEditForm] = useState<EditMeliponaryInput | null>(null);
  const [reasonAction, setReasonAction] = useState<ReasonAction>(null);
  const [search, setSearch] = useState("");
  const filtered = useMemo(() => {
    const query = search.trim().toLocaleLowerCase();
    return query ? items.filter((item) => [item.name, item.location, item.responsibleName].some((value) => value?.toLocaleLowerCase().includes(query))) : items;
  }, [items, search]);

  async function submitCreate(event: FormEvent) {
    event.preventDefault();
    if (await onCreate(form)) { setForm(initialForm); setCreateOpen(false); }
  }

  function beginEdit(item: Meliponary) {
    setEditing(item);
    setEditForm({ id: item.id, name: item.name, responsibleName: item.responsibleName || "", location: item.location || "", notes: item.notes || "", reason: "" });
  }

  async function submitEdit(event: FormEvent) {
    event.preventDefault();
    if (editForm && await onEdit(editForm)) { setEditing(null); setEditForm(null); }
  }

  async function confirmReason(reason: string) {
    if (!reasonAction) return false;
    const input = { id: reasonAction.item.id, reason };
    if (reasonAction.kind === "archive") return onArchive(input);
    if (reasonAction.kind === "reactivate") return onReactivate(input);
    return onDelete(input);
  }

  return <div className="page-stack">
    <PageToolbar title="Meliponários" description="Unidades físicas de criação e contexto operacional." count={`${items.length} cadastrados`} search={{ value: search, onChange: setSearch, placeholder: "Buscar meliponário..." }} primaryAction={{ label: "Novo meliponário", onClick: () => setCreateOpen(true), disabled: busy }} />
    <section className="panel wide-list">
      <div className="panel-heading"><h2>Locais cadastrados</h2><p>Arquivar preserva todo o histórico e impede novos vínculos operacionais. Exclusão definitiva só é aceita para cadastro nunca utilizado.</p></div>
      {filtered.length === 0 ? <div className="empty-list">{items.length === 0 ? "Nenhum meliponário cadastrado ainda." : "Nenhum resultado para a busca."}</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Nome</th><th>Localização</th><th>Responsável</th><th>Estado</th><th>Ações</th></tr></thead><tbody>{filtered.map((item) => <tr key={item.id} className={item.archivedAt ? "archived-row" : undefined}><td><strong>{item.name}</strong></td><td>{item.location || "Não informada"}</td><td>{item.responsibleName || "Não informado"}</td><td>{item.archivedAt ? <span className="badge status-archived">Arquivado</span> : <span className="badge status-active">Ativo</span>}</td><td><RecordActions busy={busy} onOpen={() => setDetail(item)} onEdit={() => beginEdit(item)} secondary={[
        item.archivedAt ? { label: "Reativar", onClick: () => setReasonAction({ kind: "reactivate", item }) } : { label: "Arquivar", onClick: () => setReasonAction({ kind: "archive", item }) },
        { label: "Excluir cadastro vazio", onClick: () => setReasonAction({ kind: "delete", item }), danger: true },
      ]} /></td></tr>)}</tbody></table></div>}
    </section>

    <Dialog open={createOpen} onClose={() => !busy && setCreateOpen(false)} title="Novo meliponário" description="Somente o nome é obrigatório." size="medium"><form className="form-grid" onSubmit={submitCreate}><label className="field full"><span>Nome</span><input autoFocus required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></label><label className="field"><span>Responsável</span><input value={form.responsibleName} onChange={(e) => setForm({ ...form, responsibleName: e.target.value })} /></label><label className="field"><span>Localização</span><input value={form.location} onChange={(e) => setForm({ ...form, location: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={4} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setCreateOpen(false)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy}>{busy ? "Salvando..." : "Salvar meliponário"}</button></div></form></Dialog>

    <Dialog open={Boolean(detail)} onClose={() => setDetail(null)} title={detail?.name || "Meliponário"} description="Ficha cadastral e estado administrativo." size="medium">{detail && <div className="detail-grid"><div><span>Responsável</span><strong>{detail.responsibleName || "Não informado"}</strong></div><div><span>Localização</span><strong>{detail.location || "Não informada"}</strong></div><div className="full"><span>Observações</span><p>{detail.notes || "Sem observações."}</p></div>{detail.archivedAt && <div className="full consequence-note"><strong>Arquivado em {detail.archivedAt}</strong><p>{detail.archiveReason || "Sem motivo informado."}</p></div>}</div>}</Dialog>

    <Dialog open={Boolean(editing && editForm)} onClose={() => { if (!busy) { setEditing(null); setEditForm(null); } }} title="Editar meliponário" description="A alteração fica registrada na auditoria." size="medium">{editForm && <form className="form-grid" onSubmit={submitEdit}><label className="field full"><span>Nome</span><input autoFocus required value={editForm.name} onChange={(e) => setEditForm({ ...editForm, name: e.target.value })} /></label><label className="field"><span>Responsável</span><input value={editForm.responsibleName} onChange={(e) => setEditForm({ ...editForm, responsibleName: e.target.value })} /></label><label className="field"><span>Localização</span><input value={editForm.location} onChange={(e) => setEditForm({ ...editForm, location: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={3} value={editForm.notes} onChange={(e) => setEditForm({ ...editForm, notes: e.target.value })} /></label><label className="field full"><span>Motivo da edição</span><textarea required rows={3} value={editForm.reason} onChange={(e) => setEditForm({ ...editForm, reason: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => { setEditing(null); setEditForm(null); }} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !editForm.reason.trim()}>Salvar alteração</button></div></form>}</Dialog>

    <ReasonDialog open={Boolean(reasonAction)} title={reasonAction?.kind === "archive" ? "Arquivar meliponário" : reasonAction?.kind === "reactivate" ? "Reativar meliponário" : "Excluir cadastro"} description={reasonAction?.item.name || ""} confirmLabel={reasonAction?.kind === "archive" ? "Arquivar" : reasonAction?.kind === "reactivate" ? "Reativar" : "Excluir definitivamente"} consequence={reasonAction?.kind === "delete" ? "A exclusão definitiva só será aceita se o cadastro nunca tiver sido usado. Se houver histórico, o backend bloqueará a operação." : "O histórico existente não será apagado."} danger={reasonAction?.kind === "delete"} busy={busy} onClose={() => setReasonAction(null)} onConfirm={confirmReason} />
  </div>;
}
