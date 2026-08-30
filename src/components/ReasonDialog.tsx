import { useEffect, useState, type FormEvent } from "react";
import { Dialog } from "./Dialog";

type ReasonDialogProps = {
  open: boolean;
  title: string;
  description: string;
  confirmLabel: string;
  busy?: boolean;
  consequence?: string;
  danger?: boolean;
  onClose: () => void;
  onConfirm: (reason: string) => Promise<boolean> | boolean;
};

export function ReasonDialog({ open, title, description, confirmLabel, busy = false, consequence, danger = false, onClose, onConfirm }: ReasonDialogProps) {
  const [reason, setReason] = useState("");
  useEffect(() => { if (!open) setReason(""); }, [open]);

  async function submit(event: FormEvent) {
    event.preventDefault();
    const value = reason.trim();
    if (!value) return;
    if (await onConfirm(value)) { setReason(""); onClose(); }
  }

  return <Dialog open={open} onClose={onClose} title={title} description={description} size="small">
    <form className="form-grid" onSubmit={submit}>
      {consequence && <div className={danger ? "consequence-note danger-note full" : "consequence-note full"}>{consequence}</div>}
      <label className="field full"><span>Motivo</span><textarea autoFocus required rows={4} value={reason} onChange={(event) => setReason(event.target.value)} placeholder="Explique por que esta operação é necessária." /></label>
      <div className="form-actions full">
        <button className="button-secondary" type="button" onClick={onClose} disabled={busy}>Cancelar</button>
        <button className={danger ? "button-danger" : undefined} type="submit" disabled={busy || !reason.trim()}>{busy ? "Processando..." : confirmLabel}</button>
      </div>
    </form>
  </Dialog>;
}
