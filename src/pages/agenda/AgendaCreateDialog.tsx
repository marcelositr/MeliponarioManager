import type { FormEventHandler } from "react";
import { Dialog } from "../../components/Dialog";
import type { TaskPriority, TaskType } from "../../lib/agenda-types";
import type { Colony, HiveBox, Meliponary } from "../../types";
import type { CreateForm } from "./forms";
import { taskTypes } from "./presentation";

type Props = {
  open: boolean;
  busy: boolean;
  form: CreateForm;
  meliponaries: Meliponary[];
  colonies: Colony[];
  boxes: HiveBox[];
  onChange: (next: CreateForm) => void;
  onClose: () => void;
  onSubmit: FormEventHandler<HTMLFormElement>;
};

export function AgendaCreateDialog({ open, busy, form, meliponaries, colonies, boxes, onChange, onClose, onSubmit }: Props) {
  return <Dialog open={open} onClose={() => !busy && onClose()} title="Nova tarefa" description="Cria um compromisso; não registra um fato como já realizado." size="large">
    <form className="form-grid" onSubmit={onSubmit}>
      <label className="field"><span>Meliponário</span><select autoFocus required value={form.meliponaryId} onChange={(event) => onChange({ ...form, meliponaryId: event.target.value, colonyId: "", boxId: "" })}>{meliponaries.map((item) => <option value={item.id} key={item.id}>{item.name}</option>)}</select></label>
      <label className="field"><span>Tipo</span><select value={form.taskType} onChange={(event) => onChange({ ...form, taskType: event.target.value as TaskType })}>{taskTypes.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label>
      <label className="field"><span>Colônia</span><select value={form.colonyId} onChange={(event) => onChange({ ...form, colonyId: event.target.value })}><option value="">Sem colônia</option>{colonies.map((item) => <option value={item.id} key={item.id}>{item.code}</option>)}</select></label>
      <label className="field"><span>Caixa</span><select value={form.boxId} onChange={(event) => onChange({ ...form, boxId: event.target.value })}><option value="">Sem caixa</option>{boxes.map((item) => <option value={item.id} key={item.id}>{item.code}</option>)}</select></label>
      <label className="field full"><span>Título</span><input required value={form.title} onChange={(event) => onChange({ ...form, title: event.target.value })} /></label>
      <label className="field full"><span>Descrição</span><textarea rows={3} value={form.description} onChange={(event) => onChange({ ...form, description: event.target.value })} /></label>
      <label className="field"><span>Data e hora</span><input required type="datetime-local" value={form.scheduledFor} onChange={(event) => onChange({ ...form, scheduledFor: event.target.value })} /></label>
      <label className="field"><span>Prioridade</span><select value={form.priority} onChange={(event) => onChange({ ...form, priority: event.target.value as TaskPriority })}><option value="normal">Normal</option><option value="attention">Atenção</option><option value="critical">Crítica</option></select></label>
      <div className="form-actions full"><button type="button" className="button-secondary" onClick={onClose} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !form.title.trim() || !form.scheduledFor}>Criar tarefa</button></div>
    </form>
  </Dialog>;
}
