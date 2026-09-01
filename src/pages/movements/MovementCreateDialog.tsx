import type { FormEventHandler } from "react";
import { Dialog } from "../../components/Dialog";
import type { Colony, CreateMovementInput, HiveBox, Meliponary } from "../../types";

type Props = {
  open: boolean;
  busy: boolean;
  transportBusy: boolean;
  movementForm: CreateMovementInput;
  selectedColonyId: string;
  colonies: Colony[];
  meliponaries: Meliponary[];
  targetBoxes: HiveBox[];
  selectedMovementColony?: Colony;
  movable: boolean;
  hasOpenTransport: boolean;
  onChange: (next: CreateMovementInput) => void;
  onClose: () => void;
  onSubmit: FormEventHandler<HTMLFormElement>;
};

export function MovementCreateDialog({
  open,
  busy,
  transportBusy,
  movementForm,
  selectedColonyId,
  colonies,
  meliponaries,
  targetBoxes,
  selectedMovementColony,
  movable,
  hasOpenTransport,
  onChange,
  onClose,
  onSubmit,
}: Props) {
  return <Dialog open={open} onClose={() => !busy && !transportBusy && onClose()} title="Nova movimentação" description="Transporte temporário não altera meliponário nem caixa atual e precisa ser concluído por um retorno." size="large">
    <form className="form-grid" onSubmit={onSubmit}>
      <label className="field full"><span>Colônia</span><select autoFocus required value={movementForm.colonyId} onChange={(event) => onChange({ ...movementForm, colonyId: event.target.value, toMeliponaryId: "", toBoxId: "" })}><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} · {colony.status}</option>)}</select></label>
      {selectedMovementColony && !movable && <div className="inline-notice field full" role="alert">Esta colônia não está disponível para nova movimentação.</div>}
      {movementForm.movementType === "transport" && selectedMovementColony?.id === selectedColonyId && hasOpenTransport && <div className="inline-notice field full" role="alert">Existe um transporte temporário aberto para esta colônia. Registre o retorno antes de iniciar outro.</div>}
      <label className="field"><span>Tipo</span><select value={movementForm.movementType} onChange={(event) => onChange({ ...movementForm, movementType: event.target.value, toMeliponaryId: "", toBoxId: "", destination: "" })}><option value="transport">Transporte temporário</option><option value="internal_transfer">Transferência interna</option><option value="external_transfer">Transferência externa</option></select></label>
      <label className="field"><span>Data e hora</span><input type="datetime-local" value={movementForm.movedAt} onChange={(event) => onChange({ ...movementForm, movedAt: event.target.value })} /></label>
      {movementForm.movementType === "internal_transfer" ? <>
        <label className="field full"><span>Meliponário de destino</span><select required value={movementForm.toMeliponaryId} onChange={(event) => onChange({ ...movementForm, toMeliponaryId: event.target.value, toBoxId: "" })}><option value="">Selecione...</option>{meliponaries.filter((item) => !item.archivedAt && item.id !== selectedMovementColony?.meliponaryId).map((item) => <option value={item.id} key={item.id}>{item.name}</option>)}</select></label>
        <label className="field full"><span>Caixa ativa e livre opcional</span><select value={movementForm.toBoxId} onChange={(event) => onChange({ ...movementForm, toBoxId: event.target.value })}><option value="">Sem caixa definida</option>{targetBoxes.map((box) => <option value={box.id} key={box.id}>{box.code}</option>)}</select></label>
      </> : <label className="field full"><span>Destino</span><input required value={movementForm.destination} onChange={(event) => onChange({ ...movementForm, destination: event.target.value })} /></label>}
      <label className="field full"><span>Observações</span><textarea rows={3} value={movementForm.notes} onChange={(event) => onChange({ ...movementForm, notes: event.target.value })} /></label>
      <div className="form-actions full"><button className="button-secondary" type="button" onClick={onClose} disabled={busy || transportBusy}>Cancelar</button><button type="submit" disabled={busy || transportBusy || !movementForm.colonyId || !movable || (movementForm.movementType === "transport" && selectedMovementColony?.id === selectedColonyId && hasOpenTransport)}>{busy ? "Salvando..." : "Registrar movimentação"}</button></div>
    </form>
  </Dialog>;
}
