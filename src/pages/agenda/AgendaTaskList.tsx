import { RecordActions } from "../../components/RecordActions";
import type { AgendaSummary, ScheduledTask, TaskView } from "../../lib/agenda-types";
import { formatDateTimeBr } from "../../lib/presentation";
import { AgendaStat, priorityLabel, statusLabel, taskTypeLabel, viewLabel } from "./presentation";

type Props = {
  summary: AgendaSummary;
  view: TaskView;
  items: ScheduledTask[];
  loading: boolean;
  busy: boolean;
  onViewChange: (view: TaskView) => void;
  onOpen: (task: ScheduledTask) => void;
  onExecute: (task: ScheduledTask) => void;
  onReschedule: (task: ScheduledTask) => void;
  onDuplicate: (task: ScheduledTask) => void;
  onSkip: (task: ScheduledTask) => void;
  onCancel: (task: ScheduledTask) => void;
};

export function AgendaTaskList({
  summary,
  view,
  items,
  loading,
  busy,
  onViewChange,
  onOpen,
  onExecute,
  onReschedule,
  onDuplicate,
  onSkip,
  onCancel,
}: Props) {
  return <>
    <section className="stats-grid executive-stats" aria-label="Resumo da Agenda">
      <AgendaStat label="Atrasadas" value={summary.overdue} attention={summary.overdue > 0} onClick={() => onViewChange("overdue")} />
      <AgendaStat label="Hoje" value={summary.today} onClick={() => onViewChange("today")} />
      <AgendaStat label="Próximos 7 dias" value={summary.nextSevenDays} onClick={() => onViewChange("upcoming")} />
      <AgendaStat label="Futuras" value={summary.future} onClick={() => onViewChange("pending")} />
    </section>

    <section className="panel wide-list">
      <div className="panel-heading"><h2>{viewLabel(view)}</h2><p>Atraso é derivado da data; concluir, cancelar, ignorar e reagendar preservam a tarefa original.</p></div>
      {loading ? <div className="empty-list" role="status">Carregando Agenda...</div> : items.length === 0 ? <div className="empty-list">Nenhuma tarefa nesta visualização.</div> : <div className="table-wrap">
        <table className="data-table">
          <thead><tr><th>Quando</th><th>Tarefa</th><th>Contexto</th><th>Tipo</th><th>Prioridade</th><th>Estado</th><th>Ações</th></tr></thead>
          <tbody>{items.map((task) => <tr key={task.id} className={task.overdue ? "attention-row" : undefined}>
            <td><strong>{formatDateTimeBr(task.scheduledFor)}</strong>{task.overdue && <small className="cell-note">Atrasada</small>}</td>
            <td><strong>{task.title}</strong>{task.description && <small className="cell-note">{task.description}</small>}</td>
            <td>{task.colonyCode || task.boxCode || task.meliponaryName}</td>
            <td>{taskTypeLabel(task.taskType)}</td>
            <td><span className={`badge severity-${task.priority}`}>{priorityLabel(task.priority)}</span></td>
            <td><span className={`badge task-${task.status}`}>{statusLabel(task.status)}</span></td>
            <td><RecordActions busy={busy} onOpen={() => onOpen(task)} secondary={task.status === "pending" ? [
              { label: task.taskType === "generic" ? "Concluir" : "Executar", onClick: () => onExecute(task) },
              { label: "Reagendar", onClick: () => onReschedule(task) },
              { label: "Duplicar", onClick: () => onDuplicate(task) },
              { label: "Ignorar", onClick: () => onSkip(task) },
              { label: "Cancelar", onClick: () => onCancel(task), danger: true },
            ] : [{ label: "Duplicar", onClick: () => onDuplicate(task) }]} /></td>
          </tr>)}</tbody>
        </table>
      </div>}
    </section>
  </>;
}
