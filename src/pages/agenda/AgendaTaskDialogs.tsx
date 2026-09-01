import type { FormEventHandler } from "react";
import { Dialog } from "../../components/Dialog";
import { ReasonDialog } from "../../components/ReasonDialog";
import type { ScheduledTask } from "../../lib/agenda-types";
import { formatDateTimeBr, linkedFactLabel } from "../../lib/presentation";
import type { ExecuteForm } from "./forms";
import { priorityLabel, statusLabel, taskTypeLabel } from "./presentation";

type Props = {
  busy: boolean;
  detail: ScheduledTask | null;
  executeTarget: ScheduledTask | null;
  executeForm: ExecuteForm;
  rescheduleTarget: ScheduledTask | null;
  rescheduleDate: string;
  rescheduleReason: string;
  duplicateTarget: ScheduledTask | null;
  duplicateDate: string;
  cancelTarget: ScheduledTask | null;
  skipTarget: ScheduledTask | null;
  onDetailChange: (next: ScheduledTask | null) => void;
  onExecuteTargetChange: (next: ScheduledTask | null) => void;
  onExecuteFormChange: (next: ExecuteForm) => void;
  onSubmitExecute: FormEventHandler<HTMLFormElement>;
  onRescheduleTargetChange: (next: ScheduledTask | null) => void;
  onRescheduleDateChange: (value: string) => void;
  onRescheduleReasonChange: (value: string) => void;
  onSubmitReschedule: FormEventHandler<HTMLFormElement>;
  onDuplicateTargetChange: (next: ScheduledTask | null) => void;
  onDuplicateDateChange: (value: string) => void;
  onSubmitDuplicate: FormEventHandler<HTMLFormElement>;
  onCancelTargetChange: (next: ScheduledTask | null) => void;
  onSkipTargetChange: (next: ScheduledTask | null) => void;
  onConfirmCancel: (reason: string) => Promise<boolean>;
  onConfirmSkip: (reason: string) => Promise<boolean>;
};

export function AgendaTaskDialogs({
  busy,
  detail,
  executeTarget,
  executeForm,
  rescheduleTarget,
  rescheduleDate,
  rescheduleReason,
  duplicateTarget,
  duplicateDate,
  cancelTarget,
  skipTarget,
  onDetailChange,
  onExecuteTargetChange,
  onExecuteFormChange,
  onSubmitExecute,
  onRescheduleTargetChange,
  onRescheduleDateChange,
  onRescheduleReasonChange,
  onSubmitReschedule,
  onDuplicateTargetChange,
  onDuplicateDateChange,
  onSubmitDuplicate,
  onCancelTargetChange,
  onSkipTargetChange,
  onConfirmCancel,
  onConfirmSkip,
}: Props) {
  return <>
    <Dialog open={Boolean(detail)} onClose={() => onDetailChange(null)} title="Tarefa" description={detail ? detail.title : ""} size="medium">
      {detail && <div className="detail-grid"><div><span>Quando</span><strong>{formatDateTimeBr(detail.scheduledFor)}</strong></div><div><span>Estado</span><strong>{statusLabel(detail.status)}</strong></div><div><span>Tipo</span><strong>{taskTypeLabel(detail.taskType)}</strong></div><div><span>Prioridade</span><strong>{priorityLabel(detail.priority)}</strong></div><div><span>Meliponário</span><strong>{detail.meliponaryName}</strong></div><div><span>Contexto</span><strong>{detail.colonyCode || detail.boxCode || "Administrativa"}</strong></div><div className="full"><span>Descrição</span><p>{detail.description || "—"}</p></div>{detail.completedById && <div className="full"><span>Fato vinculado</span><p>{linkedFactLabel(detail.completedByType)}</p></div>}{detail.cancellationReason && <div className="full"><span>Motivo do cancelamento</span><p>{detail.cancellationReason}</p></div>}{detail.skipReason && <div className="full"><span>Motivo para ignorar</span><p>{detail.skipReason}</p></div>}</div>}
    </Dialog>

    <Dialog open={Boolean(executeTarget)} onClose={() => !busy && onExecuteTargetChange(null)} title={executeTarget ? `Executar · ${executeTarget.title}` : "Executar tarefa"} description="O compromisso só será concluído depois que o fato real for salvo." size="large">
      {executeTarget && <form className="form-grid" onSubmit={onSubmitExecute}>
        <label className="field"><span>Data e hora</span><input autoFocus type="datetime-local" value={executeForm.occurredAt} onChange={(event) => onExecuteFormChange({ ...executeForm, occurredAt: event.target.value })} /></label>
        {executeTarget.taskType === "inspection" && <label className="field"><span>Força</span><select value={executeForm.strength} onChange={(event) => onExecuteFormChange({ ...executeForm, strength: event.target.value })}><option value="unknown">Sem avaliação</option><option value="strong">Forte</option><option value="medium">Média</option><option value="weak">Fraca</option></select></label>}
        {executeTarget.taskType === "feeding" && <><label className="field"><span>Alimento</span><input required value={executeForm.foodType} onChange={(event) => onExecuteFormChange({ ...executeForm, foodType: event.target.value })} /></label><label className="field"><span>Quantidade</span><input type="number" min="0" step="any" value={executeForm.quantity} onChange={(event) => onExecuteFormChange({ ...executeForm, quantity: event.target.value })} /></label><label className="field"><span>Unidade</span><input value={executeForm.unit} onChange={(event) => onExecuteFormChange({ ...executeForm, unit: event.target.value })} /></label></>}
        {executeTarget.taskType === "maintenance" && <><label className="field"><span>Tipo de manutenção</span><select value={executeForm.maintenanceType} onChange={(event) => onExecuteFormChange({ ...executeForm, maintenanceType: event.target.value })}><option value="inspection">Revisão</option><option value="cleaning">Limpeza</option><option value="repair">Reparo</option><option value="painting">Pintura</option><option value="waterproofing">Impermeabilização</option><option value="roof">Cobertura</option><option value="entrance">Entrada</option><option value="internal_structure">Estrutura interna</option><option value="other">Outra</option></select></label><label className="field full"><span>Descrição</span><textarea rows={3} value={executeForm.description} onChange={(event) => onExecuteFormChange({ ...executeForm, description: event.target.value })} /></label></>}
        <label className="field full"><span>Próximo compromisso</span><input type="datetime-local" value={executeForm.nextAt} onChange={(event) => onExecuteFormChange({ ...executeForm, nextAt: event.target.value })} /></label>
        <div className="form-actions full"><button type="button" className="button-secondary" onClick={() => onExecuteTargetChange(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || (executeTarget.taskType === "feeding" && !executeForm.foodType.trim())}>Registrar e concluir</button></div>
      </form>}
    </Dialog>

    <Dialog open={Boolean(rescheduleTarget)} onClose={() => !busy && onRescheduleTargetChange(null)} title="Reagendar tarefa" description="A tarefa original será preservada como reagendada e uma nova tarefa pendente será criada." size="medium">
      {rescheduleTarget && <form className="form-grid" onSubmit={onSubmitReschedule}><label className="field full"><span>Nova data e hora</span><input autoFocus required type="datetime-local" value={rescheduleDate} onChange={(event) => onRescheduleDateChange(event.target.value)} /></label><label className="field full"><span>Motivo</span><textarea rows={3} value={rescheduleReason} onChange={(event) => onRescheduleReasonChange(event.target.value)} /></label><div className="form-actions full"><button type="button" className="button-secondary" onClick={() => onRescheduleTargetChange(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !rescheduleDate}>Reagendar</button></div></form>}
    </Dialog>

    <Dialog open={Boolean(duplicateTarget)} onClose={() => !busy && onDuplicateTargetChange(null)} title="Duplicar tarefa" description="Copia o contexto e cria uma nova tarefa pendente, sem recorrência automática." size="small">
      {duplicateTarget && <form className="form-grid" onSubmit={onSubmitDuplicate}><label className="field full"><span>Nova data e hora</span><input autoFocus required type="datetime-local" value={duplicateDate} onChange={(event) => onDuplicateDateChange(event.target.value)} /></label><div className="form-actions full"><button type="button" className="button-secondary" onClick={() => onDuplicateTargetChange(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !duplicateDate}>Duplicar</button></div></form>}
    </Dialog>

    <ReasonDialog open={Boolean(cancelTarget)} title="Cancelar tarefa" description={cancelTarget?.title || ""} confirmLabel="Cancelar compromisso" consequence="A tarefa continuará consultável na Agenda, mas deixará de ser pendente. O motivo é obrigatório." danger busy={busy} onClose={() => onCancelTargetChange(null)} onConfirm={onConfirmCancel} />
    <ReasonDialog open={Boolean(skipTarget)} title="Ignorar tarefa" description={skipTarget?.title || ""} confirmLabel="Marcar como ignorada" consequence="Use quando o compromisso era válido, mas foi deliberadamente pulado. O motivo fica preservado." busy={busy} onClose={() => onSkipTargetChange(null)} onConfirm={onConfirmSkip} />
  </>;
}
