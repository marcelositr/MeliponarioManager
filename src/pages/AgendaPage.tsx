import { useEffect, useMemo, useRef, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { ReasonDialog } from "../components/ReasonDialog";
import { RecordActions } from "../components/RecordActions";
import {
  cancelTask,
  completeFeedingTask,
  completeGenericTask,
  completeInspectionTask,
  completeMaintenanceTask,
  createTask,
  duplicateTask,
  getAgendaSummary,
  listTasks,
  rescheduleTask,
  skipTask,
} from "../lib/agenda-api";
import type { AgendaSummary, ScheduledTask, TaskPriority, TaskType, TaskView } from "../lib/agenda-types";
import { formatDateTimeBr, linkedFactLabel, publicError } from "../lib/presentation";
import type { Colony, HiveBox, Meliponary } from "../types";

type Props = {
  meliponaries: Meliponary[];
  colonies: Colony[];
  boxes: HiveBox[];
  activeMeliponaryId: string;
  focusTaskId?: string;
  focusColonyId?: string;
};

type CreateForm = {
  meliponaryId: string;
  colonyId: string;
  boxId: string;
  taskType: TaskType;
  title: string;
  description: string;
  scheduledFor: string;
  priority: TaskPriority;
};

type ExecuteForm = {
  occurredAt: string;
  nextAt: string;
  strength: string;
  foodType: string;
  quantity: string;
  unit: string;
  maintenanceType: string;
  description: string;
};

const emptySummary: AgendaSummary = { overdue: 0, today: 0, nextSevenDays: 0, future: 0 };
const emptyExecute: ExecuteForm = { occurredAt: "", nextAt: "", strength: "unknown", foodType: "", quantity: "", unit: "", maintenanceType: "inspection", description: "" };

export function AgendaPage({ meliponaries, colonies, boxes, activeMeliponaryId, focusTaskId, focusColonyId }: Props) {
  const [items, setItems] = useState<ScheduledTask[]>([]);
  const [summary, setSummary] = useState<AgendaSummary>(emptySummary);
  const [view, setView] = useState<TaskView>("pending");
  const [taskType, setTaskType] = useState<"" | TaskType>("");
  const [priority, setPriority] = useState<"" | TaskPriority>("");
  const [colonyId, setColonyId] = useState("");
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [createOpen, setCreateOpen] = useState(false);
  const [detail, setDetail] = useState<ScheduledTask | null>(null);
  const [executeTarget, setExecuteTarget] = useState<ScheduledTask | null>(null);
  const [executeForm, setExecuteForm] = useState<ExecuteForm>(emptyExecute);
  const [rescheduleTarget, setRescheduleTarget] = useState<ScheduledTask | null>(null);
  const [rescheduleDate, setRescheduleDate] = useState("");
  const [rescheduleReason, setRescheduleReason] = useState("");
  const [cancelTarget, setCancelTarget] = useState<ScheduledTask | null>(null);
  const [skipTarget, setSkipTarget] = useState<ScheduledTask | null>(null);
  const [duplicateTarget, setDuplicateTarget] = useState<ScheduledTask | null>(null);
  const [duplicateDate, setDuplicateDate] = useState("");
  const requestSequence = useRef(0);

  const creationMeliponaries = useMemo(() => meliponaries.filter((item) => !item.archivedAt), [meliponaries]);
  const defaultMeliponaryId = activeMeliponaryId || creationMeliponaries[0]?.id || "";
  const [createForm, setCreateForm] = useState<CreateForm>(() => ({
    meliponaryId: defaultMeliponaryId,
    colonyId: "",
    boxId: "",
    taskType: "generic",
    title: "",
    description: "",
    scheduledFor: "",
    priority: "normal",
  }));

  useEffect(() => {
    if (activeMeliponaryId) setColonyId((current) => colonies.some((item) => item.id === current && item.meliponaryId === activeMeliponaryId) ? current : "");
  }, [activeMeliponaryId, colonies]);

  useEffect(() => {
    if (focusColonyId && colonies.some((item) => item.id === focusColonyId)) setColonyId(focusColonyId);
  }, [focusColonyId, colonies]);

  useEffect(() => {
    const timer = window.setTimeout(() => setDebouncedSearch(search.trim()), 250);
    return () => window.clearTimeout(timer);
  }, [search]);

  useEffect(() => { void reload(); }, [view, taskType, priority, colonyId, debouncedSearch, activeMeliponaryId]);

  useEffect(() => {
    if (!focusTaskId) return;
    let active = true;
    void listTasks({ view: "all", meliponaryId: activeMeliponaryId || undefined }).then((tasks) => {
      if (!active) return;
      const target = tasks.find((task) => task.id === focusTaskId);
      if (target) setDetail(target);
      else setError("A tarefa indicada não está disponível neste contexto.");
    }).catch((cause) => {
      if (active) setError(publicError(cause, "Não foi possível abrir a tarefa indicada."));
    });
    return () => { active = false; };
  }, [focusTaskId, activeMeliponaryId]);

  async function reload() {
    const sequence = ++requestSequence.current;
    setLoading(true);
    setError("");
    try {
      const query = {
        view,
        meliponaryId: activeMeliponaryId || undefined,
        colonyId: colonyId || undefined,
        taskType: taskType || undefined,
        priority: priority || undefined,
        search: debouncedSearch || undefined,
      };
      const [nextItems, nextSummary] = await Promise.all([
        listTasks(query),
        getAgendaSummary(activeMeliponaryId || undefined),
      ]);
      if (sequence !== requestSequence.current) return;
      setItems(nextItems);
      setSummary(nextSummary);
    } catch (cause) {
      if (sequence === requestSequence.current) setError(publicError(cause, "Não foi possível carregar a Agenda."));
    } finally {
      if (sequence === requestSequence.current) setLoading(false);
    }
  }

  async function mutate(action: () => Promise<unknown>) {
    setBusy(true);
    setError("");
    try {
      await action();
      await reload();
      return true;
    } catch (cause) {
      setError(publicError(cause, "Não foi possível concluir a ação da Agenda."));
      return false;
    } finally {
      setBusy(false);
    }
  }

  function openCreate() {
    setCreateForm({ meliponaryId: defaultMeliponaryId, colonyId: focusColonyId || "", boxId: "", taskType: "generic", title: "", description: "", scheduledFor: "", priority: "normal" });
    setCreateOpen(true);
  }

  async function submitCreate(event: FormEvent) {
    event.preventDefault();
    const ok = await mutate(() => createTask({
      meliponaryId: createForm.meliponaryId,
      colonyId: createForm.colonyId || undefined,
      boxId: createForm.boxId || undefined,
      taskType: createForm.taskType,
      title: createForm.title,
      description: createForm.description || undefined,
      scheduledFor: normalizeDateTime(createForm.scheduledFor) || createForm.scheduledFor,
      priority: createForm.priority,
    }));
    if (ok) setCreateOpen(false);
  }

  function beginExecute(task: ScheduledTask) {
    if (task.taskType === "generic") {
      void mutate(() => completeGenericTask(task.id));
      return;
    }
    setExecuteTarget(task);
    setExecuteForm(emptyExecute);
  }

  async function submitExecute(event: FormEvent) {
    event.preventDefault();
    if (!executeTarget) return;
    const occurredAt = normalizeDateTime(executeForm.occurredAt);
    const nextAt = normalizeDateTime(executeForm.nextAt);
    let action: () => Promise<unknown>;
    if (executeTarget.taskType === "inspection") {
      action = () => completeInspectionTask({ taskId: executeTarget.id, inspectedAt: occurredAt, strength: executeForm.strength, nextInspectionAt: nextAt });
    } else if (executeTarget.taskType === "feeding") {
      action = () => completeFeedingTask({ taskId: executeTarget.id, fedAt: occurredAt, foodType: executeForm.foodType, quantity: executeForm.quantity ? Number(executeForm.quantity) : undefined, unit: executeForm.unit || undefined, nextFeedingAt: nextAt });
    } else {
      action = () => completeMaintenanceTask({ taskId: executeTarget.id, maintainedAt: occurredAt, maintenanceType: executeForm.maintenanceType, description: executeForm.description || undefined, nextMaintenanceAt: nextAt });
    }
    const ok = await mutate(action);
    if (ok) setExecuteTarget(null);
  }

  async function submitReschedule(event: FormEvent) {
    event.preventDefault();
    if (!rescheduleTarget) return;
    const ok = await mutate(() => rescheduleTask({ id: rescheduleTarget.id, scheduledFor: normalizeDateTime(rescheduleDate) || rescheduleDate, reason: rescheduleReason || undefined }));
    if (ok) setRescheduleTarget(null);
  }

  async function submitDuplicate(event: FormEvent) {
    event.preventDefault();
    if (!duplicateTarget) return;
    const ok = await mutate(() => duplicateTask({ id: duplicateTarget.id, scheduledFor: normalizeDateTime(duplicateDate) || duplicateDate }));
    if (ok) setDuplicateTarget(null);
  }

  const scopedColonies = colonies.filter((item) => !activeMeliponaryId || item.meliponaryId === activeMeliponaryId);
  const createColonies = colonies.filter((item) => item.meliponaryId === createForm.meliponaryId && ["active", "weak", "recovering"].includes(item.status));
  const createBoxes = boxes.filter((item) => item.meliponaryId === createForm.meliponaryId && item.status !== "retired");

  return <div className="page-stack">
    <PageToolbar title="Agenda" description="Compromissos operacionais separados dos fatos já realizados." count={`${items.length} tarefas`} search={{ value: search, onChange: setSearch, placeholder: "Buscar título, colônia ou caixa..." }} primaryAction={{ label: "Nova tarefa", onClick: openCreate, disabled: busy || creationMeliponaries.length === 0 }}>
      <label className="toolbar-select"><span className="sr-only">Visualização</span><select value={view} onChange={(event) => setView(event.target.value as TaskView)}>{views.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label>
      <label className="toolbar-select"><span className="sr-only">Tipo</span><select value={taskType} onChange={(event) => setTaskType(event.target.value as "" | TaskType)}><option value="">Todos os tipos</option>{taskTypes.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label>
      <label className="toolbar-select"><span className="sr-only">Prioridade</span><select value={priority} onChange={(event) => setPriority(event.target.value as "" | TaskPriority)}><option value="">Todas prioridades</option><option value="normal">Normal</option><option value="attention">Atenção</option><option value="critical">Crítica</option></select></label>
      <label className="toolbar-select"><span className="sr-only">Colônia</span><select value={colonyId} onChange={(event) => setColonyId(event.target.value)}><option value="">Todas as colônias</option>{scopedColonies.map((item) => <option value={item.id} key={item.id}>{item.code}</option>)}</select></label>
    </PageToolbar>

    <section className="stats-grid executive-stats" aria-label="Resumo da Agenda">
      <AgendaStat label="Atrasadas" value={summary.overdue} attention={summary.overdue > 0} onClick={() => setView("overdue")} />
      <AgendaStat label="Hoje" value={summary.today} onClick={() => setView("today")} />
      <AgendaStat label="Próximos 7 dias" value={summary.nextSevenDays} onClick={() => setView("upcoming")} />
      <AgendaStat label="Futuras" value={summary.future} onClick={() => setView("pending")} />
    </section>

    {error && <div className="inline-notice" role="alert">{error}</div>}
    <section className="panel wide-list">
      <div className="panel-heading"><h2>{viewLabel(view)}</h2><p>Atraso é derivado da data; concluir, cancelar, ignorar e reagendar preservam a tarefa original.</p></div>
      {loading ? <div className="empty-list" role="status">Carregando Agenda...</div> : items.length === 0 ? <div className="empty-list">Nenhuma tarefa nesta visualização.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Quando</th><th>Tarefa</th><th>Contexto</th><th>Tipo</th><th>Prioridade</th><th>Estado</th><th>Ações</th></tr></thead><tbody>{items.map((task) => <tr key={task.id} className={task.overdue ? "attention-row" : undefined}><td><strong>{formatDateTimeBr(task.scheduledFor)}</strong>{task.overdue && <small className="cell-note">Atrasada</small>}</td><td><strong>{task.title}</strong>{task.description && <small className="cell-note">{task.description}</small>}</td><td>{task.colonyCode || task.boxCode || task.meliponaryName}</td><td>{taskTypeLabel(task.taskType)}</td><td><span className={`badge severity-${task.priority}`}>{priorityLabel(task.priority)}</span></td><td><span className={`badge task-${task.status}`}>{statusLabel(task.status)}</span></td><td><RecordActions busy={busy} onOpen={() => setDetail(task)} secondary={task.status === "pending" ? [
        { label: task.taskType === "generic" ? "Concluir" : "Executar", onClick: () => beginExecute(task) },
        { label: "Reagendar", onClick: () => { setRescheduleTarget(task); setRescheduleDate(toInputDateTime(task.scheduledFor)); setRescheduleReason(""); } },
        { label: "Duplicar", onClick: () => { setDuplicateTarget(task); setDuplicateDate(toInputDateTime(task.scheduledFor)); } },
        { label: "Ignorar", onClick: () => setSkipTarget(task) },
        { label: "Cancelar", onClick: () => setCancelTarget(task), danger: true },
      ] : [{ label: "Duplicar", onClick: () => { setDuplicateTarget(task); setDuplicateDate(toInputDateTime(task.scheduledFor)); } }]} /></td></tr>)}</tbody></table></div>}
    </section>

    <Dialog open={createOpen} onClose={() => !busy && setCreateOpen(false)} title="Nova tarefa" description="Cria um compromisso; não registra um fato como já realizado." size="large"><form className="form-grid" onSubmit={submitCreate}><label className="field"><span>Meliponário</span><select autoFocus required value={createForm.meliponaryId} onChange={(event) => setCreateForm({ ...createForm, meliponaryId: event.target.value, colonyId: "", boxId: "" })}>{creationMeliponaries.map((item) => <option value={item.id} key={item.id}>{item.name}</option>)}</select></label><label className="field"><span>Tipo</span><select value={createForm.taskType} onChange={(event) => setCreateForm({ ...createForm, taskType: event.target.value as TaskType })}>{taskTypes.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label><label className="field"><span>Colônia</span><select value={createForm.colonyId} onChange={(event) => setCreateForm({ ...createForm, colonyId: event.target.value })}><option value="">Sem colônia</option>{createColonies.map((item) => <option value={item.id} key={item.id}>{item.code}</option>)}</select></label><label className="field"><span>Caixa</span><select value={createForm.boxId} onChange={(event) => setCreateForm({ ...createForm, boxId: event.target.value })}><option value="">Sem caixa</option>{createBoxes.map((item) => <option value={item.id} key={item.id}>{item.code}</option>)}</select></label><label className="field full"><span>Título</span><input required value={createForm.title} onChange={(event) => setCreateForm({ ...createForm, title: event.target.value })} /></label><label className="field full"><span>Descrição</span><textarea rows={3} value={createForm.description} onChange={(event) => setCreateForm({ ...createForm, description: event.target.value })} /></label><label className="field"><span>Data e hora</span><input required type="datetime-local" value={createForm.scheduledFor} onChange={(event) => setCreateForm({ ...createForm, scheduledFor: event.target.value })} /></label><label className="field"><span>Prioridade</span><select value={createForm.priority} onChange={(event) => setCreateForm({ ...createForm, priority: event.target.value as TaskPriority })}><option value="normal">Normal</option><option value="attention">Atenção</option><option value="critical">Crítica</option></select></label><div className="form-actions full"><button type="button" className="button-secondary" onClick={() => setCreateOpen(false)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !createForm.title.trim() || !createForm.scheduledFor}>Criar tarefa</button></div></form></Dialog>

    <Dialog open={Boolean(detail)} onClose={() => setDetail(null)} title="Tarefa" description={detail ? detail.title : ""} size="medium">{detail && <div className="detail-grid"><div><span>Quando</span><strong>{formatDateTimeBr(detail.scheduledFor)}</strong></div><div><span>Estado</span><strong>{statusLabel(detail.status)}</strong></div><div><span>Tipo</span><strong>{taskTypeLabel(detail.taskType)}</strong></div><div><span>Prioridade</span><strong>{priorityLabel(detail.priority)}</strong></div><div><span>Meliponário</span><strong>{detail.meliponaryName}</strong></div><div><span>Contexto</span><strong>{detail.colonyCode || detail.boxCode || "Administrativa"}</strong></div><div className="full"><span>Descrição</span><p>{detail.description || "—"}</p></div>{detail.completedById && <div className="full"><span>Fato vinculado</span><p>{linkedFactLabel(detail.completedByType)}</p></div>}{detail.cancellationReason && <div className="full"><span>Motivo do cancelamento</span><p>{detail.cancellationReason}</p></div>}{detail.skipReason && <div className="full"><span>Motivo para ignorar</span><p>{detail.skipReason}</p></div>}</div>}</Dialog>

    <Dialog open={Boolean(executeTarget)} onClose={() => !busy && setExecuteTarget(null)} title={executeTarget ? `Executar · ${executeTarget.title}` : "Executar tarefa"} description="O compromisso só será concluído depois que o fato real for salvo." size="large">{executeTarget && <form className="form-grid" onSubmit={submitExecute}><label className="field"><span>Data e hora</span><input autoFocus type="datetime-local" value={executeForm.occurredAt} onChange={(event) => setExecuteForm({ ...executeForm, occurredAt: event.target.value })} /></label>{executeTarget.taskType === "inspection" && <label className="field"><span>Força</span><select value={executeForm.strength} onChange={(event) => setExecuteForm({ ...executeForm, strength: event.target.value })}><option value="unknown">Sem avaliação</option><option value="strong">Forte</option><option value="medium">Média</option><option value="weak">Fraca</option></select></label>}{executeTarget.taskType === "feeding" && <><label className="field"><span>Alimento</span><input required value={executeForm.foodType} onChange={(event) => setExecuteForm({ ...executeForm, foodType: event.target.value })} /></label><label className="field"><span>Quantidade</span><input type="number" min="0" step="any" value={executeForm.quantity} onChange={(event) => setExecuteForm({ ...executeForm, quantity: event.target.value })} /></label><label className="field"><span>Unidade</span><input value={executeForm.unit} onChange={(event) => setExecuteForm({ ...executeForm, unit: event.target.value })} /></label></>}{executeTarget.taskType === "maintenance" && <><label className="field"><span>Tipo de manutenção</span><select value={executeForm.maintenanceType} onChange={(event) => setExecuteForm({ ...executeForm, maintenanceType: event.target.value })}><option value="inspection">Revisão</option><option value="cleaning">Limpeza</option><option value="repair">Reparo</option><option value="painting">Pintura</option><option value="waterproofing">Impermeabilização</option><option value="roof">Cobertura</option><option value="entrance">Entrada</option><option value="internal_structure">Estrutura interna</option><option value="other">Outra</option></select></label><label className="field full"><span>Descrição</span><textarea rows={3} value={executeForm.description} onChange={(event) => setExecuteForm({ ...executeForm, description: event.target.value })} /></label></>}<label className="field full"><span>Próximo compromisso</span><input type="datetime-local" value={executeForm.nextAt} onChange={(event) => setExecuteForm({ ...executeForm, nextAt: event.target.value })} /></label><div className="form-actions full"><button type="button" className="button-secondary" onClick={() => setExecuteTarget(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || (executeTarget.taskType === "feeding" && !executeForm.foodType.trim())}>Registrar e concluir</button></div></form>}</Dialog>

    <Dialog open={Boolean(rescheduleTarget)} onClose={() => !busy && setRescheduleTarget(null)} title="Reagendar tarefa" description="A tarefa original será preservada como reagendada e uma nova tarefa pendente será criada." size="medium">{rescheduleTarget && <form className="form-grid" onSubmit={submitReschedule}><label className="field full"><span>Nova data e hora</span><input autoFocus required type="datetime-local" value={rescheduleDate} onChange={(event) => setRescheduleDate(event.target.value)} /></label><label className="field full"><span>Motivo</span><textarea rows={3} value={rescheduleReason} onChange={(event) => setRescheduleReason(event.target.value)} /></label><div className="form-actions full"><button type="button" className="button-secondary" onClick={() => setRescheduleTarget(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !rescheduleDate}>Reagendar</button></div></form>}</Dialog>

    <Dialog open={Boolean(duplicateTarget)} onClose={() => !busy && setDuplicateTarget(null)} title="Duplicar tarefa" description="Copia o contexto e cria uma nova tarefa pendente, sem recorrência automática." size="small">{duplicateTarget && <form className="form-grid" onSubmit={submitDuplicate}><label className="field full"><span>Nova data e hora</span><input autoFocus required type="datetime-local" value={duplicateDate} onChange={(event) => setDuplicateDate(event.target.value)} /></label><div className="form-actions full"><button type="button" className="button-secondary" onClick={() => setDuplicateTarget(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !duplicateDate}>Duplicar</button></div></form>}</Dialog>

    <ReasonDialog open={Boolean(cancelTarget)} title="Cancelar tarefa" description={cancelTarget?.title || ""} confirmLabel="Cancelar compromisso" consequence="A tarefa continuará consultável na Agenda, mas deixará de ser pendente. O motivo é obrigatório." danger busy={busy} onClose={() => setCancelTarget(null)} onConfirm={async (reason) => { if (!cancelTarget) return false; const ok = await mutate(() => cancelTask({ id: cancelTarget.id, reason })); if (ok) setCancelTarget(null); return ok; }} />
    <ReasonDialog open={Boolean(skipTarget)} title="Ignorar tarefa" description={skipTarget?.title || ""} confirmLabel="Marcar como ignorada" consequence="Use quando o compromisso era válido, mas foi deliberadamente pulado. O motivo fica preservado." busy={busy} onClose={() => setSkipTarget(null)} onConfirm={async (reason) => { if (!skipTarget) return false; const ok = await mutate(() => skipTask({ id: skipTarget.id, reason })); if (ok) setSkipTarget(null); return ok; }} />
  </div>;
}

const views: Array<[TaskView, string]> = [["pending", "Pendentes"], ["overdue", "Atrasadas"], ["today", "Hoje"], ["upcoming", "Próximas"], ["completed", "Concluídas"], ["cancelled", "Canceladas"], ["rescheduled", "Reagendadas"], ["skipped", "Ignoradas"], ["all", "Todas"]];
const taskTypes: Array<[TaskType, string]> = [["inspection", "Inspeção"], ["feeding", "Alimentação"], ["maintenance", "Manutenção"], ["generic", "Genérica"]];
function AgendaStat({ label, value, attention = false, onClick }: { label: string; value: number; attention?: boolean; onClick: () => void }) { return <button type="button" className={attention ? "stat-card attention stat-button" : "stat-card stat-button"} onClick={onClick}><span>{label}</span><strong>{value}</strong></button>; }
function normalizeDateTime(value?: string) { if (!value) return undefined; const normalized = value.replace("T", " "); return normalized.length === 16 ? `${normalized}:00` : normalized; }
function toInputDateTime(value: string) { return value.replace(" ", "T").slice(0, 16); }
function taskTypeLabel(value: TaskType) { return taskTypes.find(([key]) => key === value)?.[1] || value; }
function priorityLabel(value: TaskPriority) { return value === "critical" ? "Crítica" : value === "attention" ? "Atenção" : "Normal"; }
function statusLabel(value: ScheduledTask["status"]) { const labels = { pending: "Pendente", completed: "Concluída", cancelled: "Cancelada", rescheduled: "Reagendada", skipped: "Ignorada" }; return labels[value]; }
function viewLabel(value: TaskView) { return views.find(([key]) => key === value)?.[1] || "Agenda"; }
