import { useEffect, useMemo, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { ReasonDialog } from "../components/ReasonDialog";
import type { RecordStateMap } from "../hooks/useAppData";
import { listColonyMovements, listMovementDocuments } from "../lib/api";
import { formatDateTimeBr, publicError } from "../lib/presentation";
import { completeTransport, listTransportReturns, reopenTransport, type TransportReturn } from "../lib/transport-api";
import type { Colony, ColonyMovement, CorrectMovementDetailsInput, CreateMovementDocumentInput, CreateMovementInput, HiveBox, Meliponary, MovementDocument, ReverseRecordInput, UpdateMovementDocumentInput, VoidRecordInput } from "../types";
import { MovementCreateDialog } from "./movements/MovementCreateDialog";
import { MovementDocumentsDialog } from "./movements/MovementDocumentsDialog";
import { MovementHistory } from "./movements/MovementHistory";
import { movementLabel, normalizeDateTime, toInputDateTime } from "./movements/presentation";

type Props = {
  colonies: Colony[];
  meliponaries: Meliponary[];
  boxes: HiveBox[];
  busy: boolean;
  recordStateMap: RecordStateMap;
  onCreateMovement: (input: CreateMovementInput) => Promise<boolean>;
  onCreateDocument: (input: CreateMovementDocumentInput) => Promise<boolean>;
  onCorrectMovement: (input: CorrectMovementDetailsInput) => Promise<boolean>;
  onVoidTransport: (input: VoidRecordInput) => Promise<boolean>;
  onReverseMovement: (input: ReverseRecordInput) => Promise<boolean>;
  onUpdateDocument: (input: UpdateMovementDocumentInput) => Promise<boolean>;
  onVoidDocument: (input: VoidRecordInput) => Promise<boolean>;
};

type ReturnForm = { returnedAt: string; notes: string };

const movementInitial: CreateMovementInput = { colonyId: "", movementType: "transport", movedAt: "", toMeliponaryId: "", toBoxId: "", destination: "", notes: "" };
const documentInitial: CreateMovementDocumentInput = { movementId: "", documentType: "gta", referenceNumber: "", sourceSystem: "", issuer: "", issuedAt: "", validUntil: "", filePath: "", notes: "" };
const returnInitial: ReturnForm = { returnedAt: "", notes: "" };

export function MovementsPage({ colonies, meliponaries, boxes, busy, recordStateMap, onCreateMovement, onCreateDocument, onCorrectMovement, onVoidTransport, onReverseMovement, onUpdateDocument, onVoidDocument }: Props) {
  const [selectedColonyId, setSelectedColonyId] = useState("");
  const [movementForm, setMovementForm] = useState<CreateMovementInput>(movementInitial);
  const [documentForm, setDocumentForm] = useState<CreateMovementDocumentInput>(documentInitial);
  const [movements, setMovements] = useState<ColonyMovement[]>([]);
  const [transportReturns, setTransportReturns] = useState<TransportReturn[]>([]);
  const [documents, setDocuments] = useState<MovementDocument[]>([]);
  const [loading, setLoading] = useState(false);
  const [movementOpen, setMovementOpen] = useState(false);
  const [documentsOpen, setDocumentsOpen] = useState(false);
  const [movementDetail, setMovementDetail] = useState<ColonyMovement | null>(null);
  const [movementEdit, setMovementEdit] = useState<CorrectMovementDetailsInput | null>(null);
  const [movementAction, setMovementAction] = useState<{ item: ColonyMovement; mode: "void" | "reverse" } | null>(null);
  const [returnTarget, setReturnTarget] = useState<ColonyMovement | null>(null);
  const [returnForm, setReturnForm] = useState<ReturnForm>(returnInitial);
  const [reopenTarget, setReopenTarget] = useState<ColonyMovement | null>(null);
  const [transportBusy, setTransportBusy] = useState(false);
  const [transportFeedback, setTransportFeedback] = useState<{ kind: "success" | "error"; text: string } | null>(null);
  const [documentDetail, setDocumentDetail] = useState<MovementDocument | null>(null);
  const [documentEdit, setDocumentEdit] = useState<UpdateMovementDocumentInput | null>(null);
  const [documentVoid, setDocumentVoid] = useState<MovementDocument | null>(null);

  const selectedMovementColony = colonies.find((colony) => colony.id === movementForm.colonyId);
  const movable = selectedMovementColony ? !["lost", "inactive", "transferred"].includes(selectedMovementColony.status) : false;
  const targetBoxes = useMemo(() => boxes.filter((box) => box.meliponaryId === movementForm.toMeliponaryId && box.status === "active" && !box.currentColonyCode), [boxes, movementForm.toMeliponaryId]);
  const returnByMovement = useMemo(() => new Map(transportReturns.map((item) => [item.movementId, item])), [transportReturns]);
  const hasOpenTransport = useMemo(() => movements.some((item) => {
    if (item.movementType !== "transport" || returnByMovement.has(item.id)) return false;
    const state = recordStateMap.get(`movement:${item.id}`);
    return !state?.voidedAt && !state?.reversedAt;
  }), [movements, recordStateMap, returnByMovement]);

  useEffect(() => { void reloadMovements(selectedColonyId); }, [selectedColonyId]);
  useEffect(() => { void reloadDocuments(documentForm.movementId); }, [documentForm.movementId]);

  async function reloadMovements(colonyId = selectedColonyId) {
    if (!colonyId) { setMovements([]); setTransportReturns([]); return; }
    setLoading(true);
    try {
      const [nextMovements, nextReturns] = await Promise.all([listColonyMovements(colonyId), listTransportReturns(colonyId)]);
      setMovements(nextMovements);
      setTransportReturns(nextReturns);
    } catch (error) {
      setTransportFeedback({ kind: "error", text: publicError(error, "Não foi possível carregar as movimentações.") });
    } finally {
      setLoading(false);
    }
  }

  async function reloadDocuments(movementId = documentForm.movementId) {
    if (!movementId) { setDocuments([]); return; }
    setDocuments(await listMovementDocuments(movementId));
  }

  function openMovement() {
    setMovementForm({ ...movementInitial, colonyId: selectedColonyId });
    setTransportFeedback(null);
    setMovementOpen(true);
  }

  function openDocuments(movementId: string) {
    setDocumentForm({ ...documentInitial, movementId });
    setDocumentsOpen(true);
  }

  async function submitMovement(event: FormEvent) {
    event.preventDefault();
    if (movementForm.movementType === "transport" && hasOpenTransport) {
      setTransportFeedback({ kind: "error", text: "Esta colônia já possui um transporte temporário aberto. Registre o retorno antes de iniciar outro." });
      return;
    }
    const input: CreateMovementInput = {
      ...movementForm,
      movedAt: normalizeDateTime(movementForm.movedAt),
      toMeliponaryId: movementForm.movementType === "internal_transfer" ? movementForm.toMeliponaryId : undefined,
      toBoxId: movementForm.movementType === "internal_transfer" ? movementForm.toBoxId : undefined,
      destination: movementForm.movementType === "internal_transfer" ? undefined : movementForm.destination,
      documentReference: undefined,
    };
    if (await onCreateMovement(input)) {
      setSelectedColonyId(movementForm.colonyId);
      setMovementOpen(false);
      setTransportFeedback(null);
      await reloadMovements(movementForm.colonyId);
    }
  }

  async function submitReturn(event: FormEvent) {
    event.preventDefault();
    if (!returnTarget) return;
    setTransportBusy(true);
    setTransportFeedback(null);
    try {
      await completeTransport({ movementId: returnTarget.id, returnedAt: normalizeDateTime(returnForm.returnedAt), notes: returnForm.notes || undefined });
      setReturnTarget(null);
      setReturnForm(returnInitial);
      setTransportFeedback({ kind: "success", text: "Retorno registrado. O transporte temporário foi concluído sem apagar o movimento original." });
      await reloadMovements();
    } catch (error) {
      setTransportFeedback({ kind: "error", text: publicError(error, "Não foi possível registrar o retorno do transporte.") });
    } finally {
      setTransportBusy(false);
    }
  }

  async function submitDocument(event: FormEvent) {
    event.preventDefault();
    const input = { ...documentForm, issuedAt: normalizeDateTime(documentForm.issuedAt), validUntil: normalizeDateTime(documentForm.validUntil) };
    if (await onCreateDocument(input)) {
      const movementId = documentForm.movementId;
      setDocumentForm({ ...documentInitial, movementId });
      await reloadDocuments(movementId);
    }
  }

  function beginMovementEdit(item: ColonyMovement) {
    setMovementEdit({ id: item.id, reason: "", destination: item.destination || undefined, notes: item.notes || "" });
  }

  async function submitMovementEdit(event: FormEvent) {
    event.preventDefault();
    if (!movementEdit) return;
    if (await onCorrectMovement(movementEdit)) { setMovementEdit(null); await reloadMovements(); }
  }

  function beginDocumentEdit(doc: MovementDocument) {
    setDocumentEdit({ id: doc.id, reason: "", documentType: doc.documentType, referenceNumber: doc.referenceNumber, sourceSystem: doc.sourceSystem || "", issuer: doc.issuer || "", issuedAt: doc.issuedAt ? toInputDateTime(doc.issuedAt) : "", validUntil: doc.validUntil ? toInputDateTime(doc.validUntil) : "", filePath: doc.filePath || "", notes: doc.notes || "" });
  }

  async function submitDocumentEdit(event: FormEvent) {
    event.preventDefault();
    if (!documentEdit) return;
    const payload: UpdateMovementDocumentInput = { ...documentEdit, issuedAt: normalizeDateTime(documentEdit.issuedAt), validUntil: normalizeDateTime(documentEdit.validUntil) };
    if (await onUpdateDocument(payload)) { setDocumentEdit(null); await reloadDocuments(); }
  }

  return <div className="page-stack">
    <PageToolbar title="Movimentações" description="Transferências alteram localização; transporte temporário permanece aberto até o retorno ser registrado." count={selectedColonyId ? `${movements.length} movimentações` : `${colonies.length} colônias`} primaryAction={{ label: "Nova movimentação", onClick: openMovement, disabled: busy || transportBusy || colonies.length === 0 }}>
      <label className="toolbar-select"><span className="sr-only">Colônia</span><select value={selectedColonyId} onChange={(event) => setSelectedColonyId(event.target.value)}><option value="">Selecione uma colônia...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code}</option>)}</select></label>
    </PageToolbar>

    {transportFeedback && <div className={`inline-notice ${transportFeedback.kind}`} role={transportFeedback.kind === "error" ? "alert" : "status"}>{transportFeedback.text}</div>}

    <MovementHistory
      selectedColonyId={selectedColonyId}
      loading={loading}
      movements={movements}
      returnByMovement={returnByMovement}
      recordStateMap={recordStateMap}
      busy={busy}
      transportBusy={transportBusy}
      hasOpenTransport={hasOpenTransport}
      onOpenDocuments={openDocuments}
      onOpenDetail={setMovementDetail}
      onEdit={beginMovementEdit}
      onReopen={setReopenTarget}
      onReturn={(item) => { setReturnTarget(item); setReturnForm(returnInitial); }}
      onAction={(item, mode) => setMovementAction({ item, mode })}
    />

    <MovementCreateDialog
      open={movementOpen}
      busy={busy}
      transportBusy={transportBusy}
      movementForm={movementForm}
      selectedColonyId={selectedColonyId}
      colonies={colonies}
      meliponaries={meliponaries}
      targetBoxes={targetBoxes}
      selectedMovementColony={selectedMovementColony}
      movable={movable}
      hasOpenTransport={hasOpenTransport}
      onChange={setMovementForm}
      onClose={() => setMovementOpen(false)}
      onSubmit={submitMovement}
    />

    <Dialog open={Boolean(returnTarget)} onClose={() => !transportBusy && setReturnTarget(null)} title="Registrar retorno" description={returnTarget ? `${returnTarget.colonyCode} · ${returnTarget.destination || "Transporte temporário"}` : ""} size="small">
      {returnTarget && <form className="form-grid" onSubmit={submitReturn}>
        <label className="field full"><span>Data e hora do retorno</span><input autoFocus required type="datetime-local" value={returnForm.returnedAt} onChange={(event) => setReturnForm({ ...returnForm, returnedAt: event.target.value })} /></label>
        <label className="field full"><span>Observações do retorno</span><textarea rows={3} value={returnForm.notes} onChange={(event) => setReturnForm({ ...returnForm, notes: event.target.value })} placeholder="Condição da colônia, intercorrências ou observações do retorno." /></label>
        <div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setReturnTarget(null)} disabled={transportBusy}>Cancelar</button><button type="submit" disabled={transportBusy || !returnForm.returnedAt}>{transportBusy ? "Registrando..." : "Concluir transporte"}</button></div>
      </form>}
    </Dialog>

    <Dialog open={Boolean(movementDetail)} onClose={() => setMovementDetail(null)} title="Movimentação" description={movementDetail ? `${movementDetail.colonyCode} · ${formatDateTimeBr(movementDetail.movedAt)}` : ""} size="medium">
      {movementDetail && (() => { const transportReturn = returnByMovement.get(movementDetail.id); return <div className="detail-grid"><div><span>Tipo</span><strong>{movementLabel(movementDetail.movementType)}</strong></div><div><span>Origem</span><strong>{movementDetail.fromMeliponaryName}</strong></div><div><span>Caixa de origem</span><strong>{movementDetail.fromBoxCode || "—"}</strong></div><div><span>Destino</span><strong>{movementDetail.toMeliponaryName || movementDetail.destination || "—"}</strong></div><div><span>Caixa de destino</span><strong>{movementDetail.toBoxCode || "—"}</strong></div>{movementDetail.movementType === "transport" && <><div><span>Estado do transporte</span><strong>{transportReturn ? "Retornado" : "Aberto"}</strong></div><div><span>Retorno</span><strong>{transportReturn ? formatDateTimeBr(transportReturn.returnedAt) : "Pendente"}</strong></div><div className="full"><span>Observações do retorno</span><p>{transportReturn?.notes || "—"}</p></div></>}<div className="full"><span>Observações</span><p>{movementDetail.notes || "—"}</p></div></div>; })()}
    </Dialog>

    <Dialog open={Boolean(movementEdit)} onClose={() => !busy && setMovementEdit(null)} title="Corrigir dados descritivos" description="Tipo, data e relações de transferência são consequenciais e não são reescritos por esta edição." size="medium">
      {movementEdit && <form className="form-grid" onSubmit={submitMovementEdit}><label className="field full"><span>Destino textual</span><input value={movementEdit.destination || ""} onChange={(event) => setMovementEdit({ ...movementEdit, destination: event.target.value || undefined })} /></label><label className="field full"><span>Observações</span><textarea rows={3} value={movementEdit.notes || ""} onChange={(event) => setMovementEdit({ ...movementEdit, notes: event.target.value })} /></label><label className="field full"><span>Motivo da correção</span><textarea required rows={3} value={movementEdit.reason} onChange={(event) => setMovementEdit({ ...movementEdit, reason: event.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setMovementEdit(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !movementEdit.reason.trim()}>Salvar correção</button></div></form>}
    </Dialog>

    <MovementDocumentsDialog
      open={documentsOpen}
      busy={busy}
      documents={documents}
      documentForm={documentForm}
      recordStateMap={recordStateMap}
      documentDetail={documentDetail}
      documentEdit={documentEdit}
      documentVoid={documentVoid}
      onFormChange={setDocumentForm}
      onClose={() => setDocumentsOpen(false)}
      onSubmit={submitDocument}
      onDetailChange={setDocumentDetail}
      onBeginEdit={beginDocumentEdit}
      onEditChange={setDocumentEdit}
      onSubmitEdit={submitDocumentEdit}
      onVoidChange={setDocumentVoid}
      onConfirmVoid={async (reason) => {
        if (!documentVoid) return false;
        const ok = await onVoidDocument({ id: documentVoid.id, reason });
        if (ok) await reloadDocuments();
        return ok;
      }}
    />

    <ReasonDialog open={Boolean(reopenTarget)} title="Reabrir transporte temporário?" description={reopenTarget ? `${reopenTarget.colonyCode} · ${reopenTarget.destination || "Transporte"}` : ""} confirmLabel="Reabrir transporte" consequence="O retorno registrado será preservado como revertido e o transporte voltará ao estado aberto. O movimento original permanece intacto." busy={transportBusy} onClose={() => setReopenTarget(null)} onConfirm={async (reason) => { if (!reopenTarget) return false; setTransportBusy(true); setTransportFeedback(null); try { await reopenTransport({ movementId: reopenTarget.id, reason }); setTransportFeedback({ kind: "success", text: "Retorno reaberto com auditoria preservada." }); await reloadMovements(); return true; } catch (error) { setTransportFeedback({ kind: "error", text: publicError(error, "Não foi possível reabrir o transporte.") }); return false; } finally { setTransportBusy(false); } }} />
    <ReasonDialog open={Boolean(movementAction)} title={movementAction?.mode === "reverse" ? "Reverter transferência?" : "Anular transporte?"} description={movementAction ? `${movementAction.item.colonyCode} · ${formatDateTimeBr(movementAction.item.movedAt)}` : ""} confirmLabel={movementAction?.mode === "reverse" ? "Reverter transferência" : "Anular transporte"} consequence={movementAction?.mode === "reverse" ? "A reversão tenta restaurar meliponário, situação e caixa anteriores. Qualquer consequência posterior incompatível bloqueia toda a operação." : "Somente um transporte ainda aberto pode ser anulado. O registro continuará auditável, mas deixará de representar um fato operacional válido."} danger busy={busy} onClose={() => setMovementAction(null)} onConfirm={async (reason) => { if (!movementAction) return false; const payload = { id: movementAction.item.id, reason }; const ok = movementAction.mode === "reverse" ? await onReverseMovement(payload) : await onVoidTransport(payload); if (ok) await reloadMovements(); return ok; }} />
  </div>;
}
