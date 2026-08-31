import type { FormEventHandler } from "react";
import { Dialog } from "../../components/Dialog";
import { ReasonDialog } from "../../components/ReasonDialog";
import { RecordActions } from "../../components/RecordActions";
import { RecordStateBadge } from "../../components/RecordStateBadge";
import type { RecordStateMap } from "../../hooks/useAppData";
import { formatDateTimeBr } from "../../lib/presentation";
import type { CreateMovementDocumentInput, MovementDocument, UpdateMovementDocumentInput } from "../../types";
import { documentLabel, DocumentTypeOptions } from "./presentation";

type Props = {
  open: boolean;
  busy: boolean;
  documents: MovementDocument[];
  documentForm: CreateMovementDocumentInput;
  recordStateMap: RecordStateMap;
  documentDetail: MovementDocument | null;
  documentEdit: UpdateMovementDocumentInput | null;
  documentVoid: MovementDocument | null;
  onFormChange: (next: CreateMovementDocumentInput) => void;
  onClose: () => void;
  onSubmit: FormEventHandler<HTMLFormElement>;
  onDetailChange: (next: MovementDocument | null) => void;
  onBeginEdit: (doc: MovementDocument) => void;
  onEditChange: (next: UpdateMovementDocumentInput | null) => void;
  onSubmitEdit: FormEventHandler<HTMLFormElement>;
  onVoidChange: (next: MovementDocument | null) => void;
  onConfirmVoid: (reason: string) => Promise<boolean>;
};

export function MovementDocumentsDialog({
  open,
  busy,
  documents,
  documentForm,
  recordStateMap,
  documentDetail,
  documentEdit,
  documentVoid,
  onFormChange,
  onClose,
  onSubmit,
  onDetailChange,
  onBeginEdit,
  onEditChange,
  onSubmitEdit,
  onVoidChange,
  onConfirmVoid,
}: Props) {
  return <>
    <Dialog open={open} onClose={() => !busy && onClose()} title="Documentos da movimentação" description="Documentos podem ser corrigidos ou invalidados sem perder a referência histórica." size="large">
      <div className="content-grid">
        <form className="form-grid" onSubmit={onSubmit}>
          <label className="field"><span>Tipo</span><select value={documentForm.documentType} onChange={(event) => onFormChange({ ...documentForm, documentType: event.target.value })}><DocumentTypeOptions /></select></label>
          <label className="field"><span>Número / referência</span><input required value={documentForm.referenceNumber} onChange={(event) => onFormChange({ ...documentForm, referenceNumber: event.target.value })} /></label>
          <label className="field"><span>Sistema de origem</span><input value={documentForm.sourceSystem} onChange={(event) => onFormChange({ ...documentForm, sourceSystem: event.target.value })} /></label>
          <label className="field"><span>Emissor</span><input value={documentForm.issuer} onChange={(event) => onFormChange({ ...documentForm, issuer: event.target.value })} /></label>
          <label className="field"><span>Emissão</span><input type="datetime-local" value={documentForm.issuedAt} onChange={(event) => onFormChange({ ...documentForm, issuedAt: event.target.value })} /></label>
          <label className="field"><span>Validade</span><input type="datetime-local" value={documentForm.validUntil} onChange={(event) => onFormChange({ ...documentForm, validUntil: event.target.value })} /></label>
          <label className="field full"><span>Caminho do arquivo opcional</span><input value={documentForm.filePath} onChange={(event) => onFormChange({ ...documentForm, filePath: event.target.value })} /></label>
          <label className="field full"><span>Observações</span><textarea rows={2} value={documentForm.notes} onChange={(event) => onFormChange({ ...documentForm, notes: event.target.value })} /></label>
          <div className="form-actions full"><button disabled={busy} type="submit">{busy ? "Salvando..." : "Vincular documento"}</button></div>
        </form>
        <div>{documents.length === 0 ? <div className="empty-list">Nenhum documento vinculado.</div> : <div className="record-list">{documents.map((doc) => {
          const state = recordStateMap.get(`movement_document:${doc.id}`);
          return <article className={state?.voidedAt ? "record-card voided-row" : "record-card"} key={doc.id}>
            <div className="record-title-row"><div><strong>{documentLabel(doc.documentType)} · {doc.referenceNumber}</strong><span>{doc.sourceSystem || doc.issuer || "Sem emissor informado"}</span></div><RecordStateBadge state={state} /></div>
            {doc.notes && <p>{doc.notes}</p>}
            <RecordActions busy={busy} onOpen={() => onDetailChange(doc)} onEdit={state?.voidedAt ? undefined : () => onBeginEdit(doc)} secondary={[{ label: "Invalidar", onClick: () => onVoidChange(doc), disabled: Boolean(state?.voidedAt), danger: true }]} />
          </article>;
        })}</div>}</div>
      </div>
    </Dialog>

    <Dialog open={Boolean(documentDetail)} onClose={() => onDetailChange(null)} title="Documento" description={documentDetail ? `${documentLabel(documentDetail.documentType)} · ${documentDetail.referenceNumber}` : ""} size="medium">
      {documentDetail && <div className="detail-grid"><div><span>Sistema</span><strong>{documentDetail.sourceSystem || "—"}</strong></div><div><span>Emissor</span><strong>{documentDetail.issuer || "—"}</strong></div><div><span>Emissão</span><strong>{documentDetail.issuedAt ? formatDateTimeBr(documentDetail.issuedAt) : "—"}</strong></div><div><span>Validade</span><strong>{documentDetail.validUntil ? formatDateTimeBr(documentDetail.validUntil) : "—"}</strong></div><div className="full"><span>Arquivo</span><p>{documentDetail.filePath || "—"}</p></div><div className="full"><span>Observações</span><p>{documentDetail.notes || "—"}</p></div></div>}
    </Dialog>

    <Dialog open={Boolean(documentEdit)} onClose={() => !busy && onEditChange(null)} title="Editar documento" description="A versão anterior permanece na auditoria." size="large">
      {documentEdit && <form className="form-grid" onSubmit={onSubmitEdit}>
        <label className="field"><span>Tipo</span><select value={documentEdit.documentType} onChange={(event) => onEditChange({ ...documentEdit, documentType: event.target.value })}><DocumentTypeOptions /></select></label>
        <label className="field"><span>Número / referência</span><input required value={documentEdit.referenceNumber} onChange={(event) => onEditChange({ ...documentEdit, referenceNumber: event.target.value })} /></label>
        <label className="field"><span>Sistema de origem</span><input value={documentEdit.sourceSystem || ""} onChange={(event) => onEditChange({ ...documentEdit, sourceSystem: event.target.value })} /></label>
        <label className="field"><span>Emissor</span><input value={documentEdit.issuer || ""} onChange={(event) => onEditChange({ ...documentEdit, issuer: event.target.value })} /></label>
        <label className="field"><span>Emissão</span><input type="datetime-local" value={documentEdit.issuedAt || ""} onChange={(event) => onEditChange({ ...documentEdit, issuedAt: event.target.value })} /></label>
        <label className="field"><span>Validade</span><input type="datetime-local" value={documentEdit.validUntil || ""} onChange={(event) => onEditChange({ ...documentEdit, validUntil: event.target.value })} /></label>
        <label className="field full"><span>Caminho do arquivo</span><input value={documentEdit.filePath || ""} onChange={(event) => onEditChange({ ...documentEdit, filePath: event.target.value })} /></label>
        <label className="field full"><span>Observações</span><textarea rows={2} value={documentEdit.notes || ""} onChange={(event) => onEditChange({ ...documentEdit, notes: event.target.value })} /></label>
        <label className="field full"><span>Motivo da edição</span><textarea required rows={3} value={documentEdit.reason} onChange={(event) => onEditChange({ ...documentEdit, reason: event.target.value })} /></label>
        <div className="form-actions full"><button className="button-secondary" type="button" onClick={() => onEditChange(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !documentEdit.reason.trim() || !documentEdit.referenceNumber.trim()}>Salvar documento</button></div>
      </form>}
    </Dialog>

    <ReasonDialog open={Boolean(documentVoid)} title="Invalidar documento?" description={documentVoid ? `${documentLabel(documentVoid.documentType)} · ${documentVoid.referenceNumber}` : ""} confirmLabel="Invalidar documento" consequence="O vínculo e a referência permanecerão preservados na auditoria, mas o documento deixará de ser considerado válido." danger busy={busy} onClose={() => onVoidChange(null)} onConfirm={onConfirmVoid} />
  </>;
}
