import { useEffect, useMemo, useRef, useState, type FormEvent } from "react";
import { PageToolbar } from "../components/PageToolbar";
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
import { publicError } from "../lib/presentation";
import type { Colony, HiveBox, Meliponary } from "../types";
import { AgendaCreateDialog } from "./agenda/AgendaCreateDialog";
import { AgendaTaskDialogs } from "./agenda/AgendaTaskDialogs";
import { AgendaTaskList } from "./agenda/AgendaTaskList";
import { emptyExecute, type CreateForm, type ExecuteForm } from "./agenda/forms";
import { normalizeDateTime, taskTypes, toInputDateTime, views } from "./agenda/presentation";

type Props = {
  meliponaries: Meliponary[];
  colonies: Colony[];
  boxes: HiveBox[];
  activeMeliponaryId: string;
  focusTaskId?: string;
  focusColonyId?: string;
};

const emptySummary: AgendaSummary = { overdue: 0, today: 0, nextSevenDays: 0, future: 0 };

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

    {error && <div className="inline-notice" role="alert">{error}</div>}

    <AgendaTaskList
      summary={summary}
      view={view}
      items={items}
      loading={loading}
      busy={busy}
      onViewChange={setView}
      onOpen={setDetail}
      onExecute={beginExecute}
      onReschedule={(task) => { setRescheduleTarget(task); setRescheduleDate(toInputDateTime(task.scheduledFor)); setRescheduleReason(""); }}
      onDuplicate={(task) => { setDuplicateTarget(task); setDuplicateDate(toInputDateTime(task.scheduledFor)); }}
      onSkip={setSkipTarget}
      onCancel={setCancelTarget}
    />

    <AgendaCreateDialog
      open={createOpen}
      busy={busy}
      form={createForm}
      meliponaries={creationMeliponaries}
      colonies={createColonies}
      boxes={createBoxes}
      onChange={setCreateForm}
      onClose={() => setCreateOpen(false)}
      onSubmit={submitCreate}
    />

    <AgendaTaskDialogs
      busy={busy}
      detail={detail}
      executeTarget={executeTarget}
      executeForm={executeForm}
      rescheduleTarget={rescheduleTarget}
      rescheduleDate={rescheduleDate}
      rescheduleReason={rescheduleReason}
      duplicateTarget={duplicateTarget}
      duplicateDate={duplicateDate}
      cancelTarget={cancelTarget}
      skipTarget={skipTarget}
      onDetailChange={setDetail}
      onExecuteTargetChange={setExecuteTarget}
      onExecuteFormChange={setExecuteForm}
      onSubmitExecute={submitExecute}
      onRescheduleTargetChange={setRescheduleTarget}
      onRescheduleDateChange={setRescheduleDate}
      onRescheduleReasonChange={setRescheduleReason}
      onSubmitReschedule={submitReschedule}
      onDuplicateTargetChange={setDuplicateTarget}
      onDuplicateDateChange={setDuplicateDate}
      onSubmitDuplicate={submitDuplicate}
      onCancelTargetChange={setCancelTarget}
      onSkipTargetChange={setSkipTarget}
      onConfirmCancel={async (reason) => {
        if (!cancelTarget) return false;
        const ok = await mutate(() => cancelTask({ id: cancelTarget.id, reason }));
        if (ok) setCancelTarget(null);
        return ok;
      }}
      onConfirmSkip={async (reason) => {
        if (!skipTarget) return false;
        const ok = await mutate(() => skipTask({ id: skipTarget.id, reason }));
        if (ok) setSkipTarget(null);
        return ok;
      }}
    />
  </div>;
}
