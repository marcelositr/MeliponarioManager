import { useMemo, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { BoxOperationalCenter } from "../components/OperationalRecordCenter";
import { PageToolbar } from "../components/PageToolbar";
import { ReasonDialog } from "../components/ReasonDialog";
import { RecordActions } from "../components/RecordActions";
import { listBoxStateHistory } from "../lib/api";
import type { Navigate } from "../lib/navigation";
import type { BoxStateRecord, ChangeBoxStateInput, CreateBoxInput, EditBoxInput, EntityActionInput, HiveBox, Meliponary } from "../types";

type Props = {
  items: HiveBox[]; meliponaries: Meliponary[]; busy: boolean;
  onCreate: (input: CreateBoxInput) => Promise<boolean>;
  onEdit: (input: EditBoxInput) => Promise<boolean>;
  onChangeState: (input: ChangeBoxStateInput) => Promise<boolean>;
  onDelete: (input: EntityActionInput) => Promise<boolean>;
  onNavigate: Navigate;
};
const initialForm: CreateBoxInput = { meliponaryId: "", code: "", model: "", material: "", locationNote: "", notes: "" };
type ReasonAction = { kind: "state" | "delete"; item: HiveBox; nextState?: string } | null;

export function BoxesPage({ items, meliponaries, busy, onCreate, onEdit, onChangeState, onDelete, onNavigate }: Props) {
  const [form, setForm] = useState<CreateBoxInput>(initialForm);
  const [createOpen, setCreateOpen] = useState(false);
  const [detail, setDetail] = useState<HiveBox | null>(null);
  const [history, setHistory] = useState<BoxStateRecord[]>([]);
  const [editForm, setEditForm] = useState<EditBoxInput | null>(null);
  const [reasonAction, setReasonAction] = useState<ReasonAction>(null);
  const [search, setSearch] = useState("");
  const [status, setStatus] = useState("all");
  const activeMeliponaries = useMemo(() => meliponaries.filter((item) => !item.archivedAt), [meliponaries]);
  const meliponaryNames = useMemo(() => new Map(meliponaries.map((item) => [item.id, item.name])), [meliponaries]);
  const filtered = useMemo(() => { const query = search.trim().toLocaleLowerCase(); return items.filter((item) => (status === "all" || item.status === status) && (!query || [item.code, item.model, item.material, item.locationNote, item.currentColonyCode, meliponaryNames.get(item.meliponaryId)].some((value) => value?.toLocaleLowerCase().includes(query)))); }, [items, meliponaryNames, search, status]);

  async function submitCreate(event: FormEvent) { event.preventDefault(); if (await onCreate(form)) { setForm({ ...initialForm, meliponaryId: form.meliponaryId }); setCreateOpen(false); } }
  function beginEdit(item: HiveBox) { setEditForm({ id: item.id, code: item.code, model: item.model || "", material: item.material || "", locationNote: item.locationNote || "", notes: item.notes || "", reason: "" }); }
  async function submitEdit(event: FormEvent) { event.preventDefault(); if (editForm && await onEdit(editForm)) setEditForm(null); }
  async function openDetail(item: HiveBox) { setDetail(item); try { setHistory(await listBoxStateHistory(item.id)); } catch { setHistory([]); } }
  async function confirmReason(reason: string) {
    if (!reasonAction) return false;
    if (reasonAction.kind === "delete") return onDelete({ id: reasonAction.item.id, reason });
    return onChangeState({ boxId: reasonAction.item.id, newStatus: reasonAction.nextState || "active", reason });
  }

  return <div className="page-stack">
    <PageToolbar title="Caixas" description="Estrutura física separada da identidade das colônias." count={`${items.length} cadastradas`} search={{ value: search, onChange: setSearch, placeholder: "Buscar caixa..." }} primaryAction={{ label: "Nova caixa", onClick: () => setCreateOpen(true), disabled: busy || activeMeliponaries.length === 0 }}><label className="toolbar-select"><span className="sr-only">Estado</span><select value={status} onChange={(e) => setStatus(e.target.value)}><option value="all">Todos os estados</option><option value="active">Ativas</option><option value="maintenance">Manutenção</option><option value="retired">Aposentadas</option></select></label></PageToolbar>
    {activeMeliponaries.length === 0 && <div className="inline-notice">Cadastre ou reative um meliponário antes de adicionar caixas.</div>}
    <section className="panel wide-list"><div className="panel-heading"><h2>Caixas cadastradas</h2><p>Estado físico possui transições próprias. Caixa ocupada não pode entrar em manutenção nem ser aposentada.</p></div>{filtered.length === 0 ? <div className="empty-list">{items.length === 0 ? "Nenhuma caixa cadastrada ainda." : "Nenhuma caixa corresponde aos filtros."}</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Código</th><th>Meliponário</th><th>Colônia atual</th><th>Estado</th><th>Modelo</th><th>Ações</th></tr></thead><tbody>{filtered.map((item) => {
      const secondary = [] as { label: string; onClick: () => void; danger?: boolean }[];
      if (!item.currentColonyCode && item.status === "active") secondary.push({ label: "Entrar em manutenção", onClick: () => setReasonAction({ kind: "state", item, nextState: "maintenance" }) }, { label: "Aposentar", onClick: () => setReasonAction({ kind: "state", item, nextState: "retired" }) });
      if (item.status === "maintenance") secondary.push({ label: "Voltar a ativa", onClick: () => setReasonAction({ kind: "state", item, nextState: "active" }) }, { label: "Aposentar", onClick: () => setReasonAction({ kind: "state", item, nextState: "retired" }) });
      secondary.push({ label: "Excluir cadastro vazio", onClick: () => setReasonAction({ kind: "delete", item }), danger: true });
      return <tr key={item.id}><td><strong>{item.code}</strong></td><td>{meliponaryNames.get(item.meliponaryId) || "Meliponário"}</td><td>{item.currentColonyCode || "Livre"}</td><td><span className={`badge status-${item.status}`}>{boxStatusLabel(item.status)}</span></td><td>{item.model || "—"}</td><td><RecordActions busy={busy} onOpen={() => { void openDetail(item); }} onEdit={() => beginEdit(item)} secondary={secondary} /></td></tr>;
    })}</tbody></table></div>}</section>

    <Dialog open={createOpen} onClose={() => !busy && setCreateOpen(false)} title="Nova caixa" description="O código precisa ser único dentro do meliponário." size="medium"><form className="form-grid" onSubmit={submitCreate}><label className="field full"><span>Meliponário</span><select autoFocus required value={form.meliponaryId} onChange={(e) => setForm({ ...form, meliponaryId: e.target.value })}><option value="">Selecione...</option>{activeMeliponaries.map((item) => <option value={item.id} key={item.id}>{item.name}</option>)}</select></label><label className="field"><span>Código</span><input required value={form.code} onChange={(e) => setForm({ ...form, code: e.target.value })} /></label><label className="field"><span>Modelo</span><input value={form.model} onChange={(e) => setForm({ ...form, model: e.target.value })} /></label><label className="field"><span>Material</span><input value={form.material} onChange={(e) => setForm({ ...form, material: e.target.value })} /></label><label className="field"><span>Posição / local</span><input value={form.locationNote} onChange={(e) => setForm({ ...form, locationNote: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={3} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setCreateOpen(false)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy}>Salvar caixa</button></div></form></Dialog>

    <Dialog open={Boolean(detail)} onClose={() => { setDetail(null); setHistory([]); }} title={detail?.code || "Caixa"} description="Ficha física, histórico de estado e centro operacional." size="large">{detail && <div className="page-stack compact-stack"><div className="detail-grid"><div><span>Meliponário</span><strong>{meliponaryNames.get(detail.meliponaryId) || "—"}</strong></div><div><span>Estado</span><strong>{boxStatusLabel(detail.status)}</strong></div><div><span>Colônia atual</span><strong>{detail.currentColonyCode || "Livre"}</strong></div><div><span>Modelo</span><strong>{detail.model || "—"}</strong></div><div><span>Material</span><strong>{detail.material || "—"}</strong></div><div><span>Posição</span><strong>{detail.locationNote || "—"}</strong></div></div><div><h3>Histórico de estado</h3>{history.length === 0 ? <p className="muted">Nenhuma transição registrada.</p> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Data</th><th>De</th><th>Para</th><th>Motivo</th></tr></thead><tbody>{history.map((row) => <tr key={row.id}><td>{row.occurredAt}</td><td>{boxStatusLabel(row.previousStatus)}</td><td>{boxStatusLabel(row.newStatus)}</td><td>{row.reason || "—"}</td></tr>)}</tbody></table></div>}</div><BoxOperationalCenter boxId={detail.id} onNavigate={onNavigate} /></div>}</Dialog>

    <Dialog open={Boolean(editForm)} onClose={() => !busy && setEditForm(null)} title="Editar caixa" description="Somente dados descritivos e código seguro. Estado físico usa o fluxo próprio." size="medium">{editForm && <form className="form-grid" onSubmit={submitEdit}><label className="field"><span>Código</span><input autoFocus required value={editForm.code} onChange={(e) => setEditForm({ ...editForm, code: e.target.value })} /></label><label className="field"><span>Modelo</span><input value={editForm.model} onChange={(e) => setEditForm({ ...editForm, model: e.target.value })} /></label><label className="field"><span>Material</span><input value={editForm.material} onChange={(e) => setEditForm({ ...editForm, material: e.target.value })} /></label><label className="field"><span>Posição / local</span><input value={editForm.locationNote} onChange={(e) => setEditForm({ ...editForm, locationNote: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={3} value={editForm.notes} onChange={(e) => setEditForm({ ...editForm, notes: e.target.value })} /></label><label className="field full"><span>Motivo da edição</span><textarea required rows={3} value={editForm.reason} onChange={(e) => setEditForm({ ...editForm, reason: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setEditForm(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !editForm.reason.trim()}>Salvar alteração</button></div></form>}</Dialog>

    <ReasonDialog open={Boolean(reasonAction)} title={reasonAction?.kind === "delete" ? "Excluir caixa" : reasonAction?.nextState === "maintenance" ? "Enviar caixa para manutenção" : reasonAction?.nextState === "retired" ? "Aposentar caixa" : "Reativar caixa"} description={reasonAction?.item.code || ""} confirmLabel={reasonAction?.kind === "delete" ? "Excluir definitivamente" : "Confirmar estado"} consequence={reasonAction?.kind === "delete" ? "Só uma caixa nunca utilizada pode ser apagada. Qualquer histórico bloqueia a exclusão." : reasonAction?.nextState === "retired" ? "Aposentadoria é terminal nesta etapa e exige que a caixa esteja vazia." : "A mudança será registrada no histórico físico da caixa."} danger={reasonAction?.kind === "delete" || reasonAction?.nextState === "retired"} busy={busy} onClose={() => setReasonAction(null)} onConfirm={confirmReason} />
  </div>;
}

function boxStatusLabel(status: string) { const labels: Record<string, string> = { active: "Ativa", maintenance: "Manutenção", retired: "Aposentada" }; return labels[status] || status; }
