import { Dialog } from "./Dialog";

type ConfirmDialogProps = {
  open: boolean;
  title: string;
  consequence: string;
  confirmLabel: string;
  cancelLabel?: string;
  danger?: boolean;
  busy?: boolean;
  onCancel: () => void;
  onConfirm: () => void;
};

export function ConfirmDialog({ open, title, consequence, confirmLabel, cancelLabel = "Cancelar", danger = false, busy = false, onCancel, onConfirm }: ConfirmDialogProps) {
  const footer = <div className="dialog-actions">
    <button className="button-secondary" type="button" onClick={onCancel} disabled={busy}>{cancelLabel}</button>
    <button className={danger ? "button-danger" : undefined} type="button" onClick={onConfirm} disabled={busy}>{busy ? "Processando..." : confirmLabel}</button>
  </div>;

  return <Dialog open={open} title={title} onClose={onCancel} size="small" footer={footer}>
    <div className="confirm-dialog"><p>{consequence}</p></div>
  </Dialog>;
}
