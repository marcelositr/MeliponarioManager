import { useEffect, useMemo, useState, type FormEvent } from "react";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { ReasonDialog } from "../components/ReasonDialog";
import { RecordActions } from "../components/RecordActions";
import { RecordStateBadge } from "../components/RecordStateBadge";
import type { RecordStateMap } from "../hooks/useAppData";
import { listColonyMovements, listMovementDocuments } from "../lib/api";
import { formatDateTimeBr, publicError } from "../lib/presentation";
import { completeTransport, listTransportReturns, reopenTransport, type TransportReturn } from "../lib/transport-api";
import type { Colony, ColonyMovement, CorrectMovementDetailsInput, CreateMovementDocumentInput, CreateMovementInput, HiveBox, Meliponary, MovementDocument, ReverseRecordInput, UpdateMovementDocumentInput, VoidRecordInput } from "../types";

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

    <section className="panel wide-list">
      <div className="panel-heading"><h2>Histórico da colônia</h2><p>O movimento de saída e o retorno são fatos separados. Reabrir um retorno preserva o registro anterior e a auditoria.</p></div>
      {!selectedColonyId ? <div className="empty-list">Selecione uma colônia na toolbar.</div> : loading ? <div className="empty-list">Carregando...</div> : movements.length === 0 ? <div className="empty-list">Nenhuma movimentação registrada.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Data</th><th>Tipo</th><th>Origem</th><th>Destino</th><th>Estado</th><th>Ações</th></tr></thead><tbody>{movements.map((item) => {
        const state = recordStateMap.get(`movement:${item.id}`);
        const disabled = Boolean(state?.voidedAt || state?.reversedAt);
        const transportReturn = returnByMovement.get(item.id);
        const secondary = [{ label: "Documentos", onClick: () => openDocuments(item.id) }];
        if (!disabled && item.movementType === "transport") {
          if (transportReturn) {
            if (!hasOpenTransport) secondary.push({ label: "Reabrir transporte…", onClick: () => setReopenTarget(item) });
          } else {
            secondary.push({ label: "Registrar retorno…", onClick: () => { setReturnTarget(item); setReturnForm(returnInitial); } });
            secondary.push({ label: "Anular transporte", onClick: () => setMovementAction({ item, mode: "void" }) });
          }
        } else if (!disabled && item.movementType !== "transport") {
          secondary.push({ label: "Reverter transferência", onClick: () => setMovementAction({ item, mode: "reverse" }) });
        }
        return <tr key={item.id} className={disabled ? "voided-row" : undefined}>
          <td><strong>{formatDateTimeBr(item.movedAt)}</strong></td>
          <td>{movementLabel(item.movementType)}</td>
          <td>{item.fromMeliponaryName}</td>
          <td>{item.toMeliponaryName || item.destination || "—"}</td>
          <td>{item.movementType === "transport" && !disabled ? transportReturn ? <><span className="badge status-active">Retornado</span><small className="cell-note">{formatDateTimeBr(transportReturn.returnedAt)}</small></> : <span className="badge severity-attention">Transporte aberto</span> : <RecordStateBadge state={state} />}</td>
          <td><RecordActions busy={busy || transportBusy} onOpen={() => setMovementDetail(item)} onEdit={disabled ? undefined : () => beginMovementEdit(item)} secondary={secondary} /></td>
        </tr>;
      })}</tbody></table></div>}
    </section>

    <Dialog open={movementOpen} onClose={() => !busy && !transportBusy && setMovementOpen(false)} title="Nova movimentação" description="Transporte temporário não altera meliponário nem caixa atual e precisa ser concluído por um retorno." size="large">
      <form className="form-grid" onSubmit={submitMovement}>
        <label className="field full"><span>Colônia</span><select autoFocus required value={movementForm.colonyId} onChange={(event) => setMovementForm({ ...movementForm, colonyId: event.target.value, toMeliponaryId: "", toBoxId: "" })}><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} · {colony.status}</option>)}</select></label>
        {selectedMovementColony && !movable && <div className="inline-notice field full" role="alert">Esta colônia não está disponível para nova movimentação.</div>}
        {movementForm.movementType === "transport" && selectedMovementColony?.id === selectedColonyId && hasOpenTransport && <div className="inline-notice field full" role="alert">Existe um transporte temporário aberto para esta colônia. Registre o retorno antes de iniciar outro.</div>}
        <label className="field"><span>Tipo</span><select value={movementForm.movementType} onChange={(event) => setMovementForm({ ...movementForm, movementType: event.target.value, toMeliponaryId: "", toBoxId: "", destination: "" })}><option value="transport">Transporte temporário</option><option value="internal_transfer">Transferência interna</option><option value="external_transfer">Transferência externa</option></select></label>
        <label className="field"><span>Data e hora</span><input type="datetime-local" value={movementForm.movedAt} onChange={(event) => setMovementForm({ ...movementForm, movedAt: event.target.value })} /></label>
        {movementForm.movementType === "internal_transfer" ? <>
          <label className="field full"><span>Meliponário de destino</span><select required value={movementForm.toMeliponaryId} onChange={(event) => setMovementForm({ ...movementForm, toMeliponaryId: event.target.value, toBoxId: "" })}><option value="">Selecione...</option>{meliponaries.filter((item) => !item.archivedAt && item.id !== selectedMovementColony?.meliponaryId).map((item) => <option value={item.id} key={item.id}>{item.name}</option>)}</select></label>
          <label className="field full"><span>Caixa ativa e livre opcional</span><select value={movementForm.toBoxId} onChange={(event) => setMovementForm({ ...movementForm, toBoxId: event.target.value })}><option value="">Sem caixa definida</option>{targetBoxes.map((box) => <option value={box.id} key={box.id}>{box.code}</option>)}</select></label>
        </> : <label className="field full"><span>Destino</span><input required value={movementForm.destination} onChange={(event) => setMovementForm({ ...movementForm, destination: event.target.value })} /></label>}
        <label className="field full"><span>Observações</span><textarea rows={3} value={movementForm.notes} onChange={(event) => setMovementForm({ ...movementForm, notes: event.target.value })} /></label>
        <div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setMovementOpen(false)} disabled={busy || transportBusy}>Cancelar</button><button type="submit" disabled={busy || transportBusy || !movementForm.colonyId || !movable || (movementForm.movementType === "transport" && selectedMovementColony?.id === selectedColonyId && hasOpenTransport)}>{busy ? "Salvando..." : "Registrar movimentação"}</button></div>
      </form>
    </Dialog>

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

    <Dialog open={documentsOpen} onClose={() => !busy && setDocumentsOpen(false)} title="Documentos da movimentação" description="Documentos podem ser corrigidos ou invalidados sem perder a referência histórica." size="large">
      <div className="content-grid">
        <form className="form-grid" onSubmit={submitDocument}><label className="field"><span>Tipo</span><select value={documentForm.documentType} onChange={(event) => setDocumentForm({ ...documentForm, documentType: event.target.value })}>{documentTypeOptions()}</select></label><label className="field"><span>Número / referência</span><input required value={documentForm.referenceNumber} onChange={(event) => setDocumentForm({ ...documentForm, referenceNumber: event.target.value })} /></label><label className="field"><span>Sistema de origem</span><input value={documentForm.sourceSystem} onChange={(event) => setDocumentForm({ ...documentForm, sourceSystem: event.target.value })} /></label><label className="field"><span>Emissor</span><input value={documentForm.issuer} onChange={(event) => setDocumentForm({ ...documentForm, issuer: event.target.value })} /></label><label className="field"><span>Emissão</span><input type="datetime-local" value={documentForm.issuedAt} onChange={(event) => setDocumentForm({ ...documentForm, issuedAt: event.target.value })} /></label><label className="field"><span>Validade</span><input type="datetime-local" value={documentForm.validUntil} onChange={(event) => setDocumentForm({ ...documentForm, validUntil: event.target.value })} /></label><label className="field full"><span>Caminho do arquivo opcional</span><input value={documentForm.filePath} onChange={(event) => setDocumentForm({ ...documentForm, filePath: event.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={2} value={documentForm.notes} onChange={(event) => setDocumentForm({ ...documentForm, notes: event.target.value })} /></label><div className="form-actions full"><button disabled={busy} type="submit">{busy ? "Salvando..." : "Vincular documento"}</button></div></form>
        <div>{documents.length === 0 ? <div className="empty-list">Nenhum documento vinculado.</div> : <div className="record-list">{documents.map((doc) => { const state = recordStateMap.get(`movement_document:${doc.id}`); return <article className={state?.voidedAt ? "record-card voided-row" : "record-card"} key={doc.id}><div className="record-title-row"><div><strong>{documentLabel(doc.documentType)} · {doc.referenceNumber}</strong><span>{doc.sourceSystem || doc.issuer || "Sem emissor informado"}</span></div><RecordStateBadge state={state} /></div>{doc.notes && <p>{doc.notes}</p>}<RecordActions busy={busy} onOpen={() => setDocumentDetail(doc)} onEdit={state?.voidedAt ? undefined : () => beginDocumentEdit(doc)} secondary={[{ label: "Invalidar", onClick: () => setDocumentVoid(doc), disabled: Boolean(state?.voidedAt), danger: true }]} /></article>; })}</div>}</div>
      </div>
    </Dialog>

    <Dialog open={Boolean(documentDetail)} onClose={() => setDocumentDetail(null)} title="Documento" description={documentDetail ? `${documentLabel(documentDetail.documentType)} · ${documentDetail.referenceNumber}` : ""} size="medium">
      {documentDetail && <div className="detail-grid"><div><span>Sistema</span><strong>{documentDetail.sourceSystem || "—"}</strong></div><div><span>Emissor</span><strong>{documentDetail.issuer || "—"}</strong></div><div><span>Emissão</span><strong>{documentDetail.issuedAt ? formatDateTimeBr(documentDetail.issuedAt) : "—"}</strong></div><div><span>Validade</span><strong>{documentDetail.validUntil ? formatDateTimeBr(documentDetail.validUntil) : "—"}</strong></div><div className="full"><span>Arquivo</span><p>{documentDetail.filePath || "—"}</p></div><div className="full"><span>Observações</span><p>{documentDetail.notes || "—"}</p></div></div>}
    </Dialog>

    <Dialog open={Boolean(documentEdit)} onClose={() => !busy && setDocumentEdit(null)} title="Editar documento" description="A versão anterior permanece na auditoria." size="large">
      {documentEdit && <form className="form-grid" onSubmit={submitDocumentEdit}><label className="field"><span>Tipo</span><select value={documentEdit.documentType} onChange={(event) => setDocumentEdit({ ...documentEdit, documentType: event.target.value })}>{documentTypeOptions()}</select></label><label className="field"><span>Número / referência</span><input required value={documentEdit.referenceNumber} onChange={(event) => setDocumentEdit({ ...documentEdit, referenceNumber: event.target.value })} /></label><label className="field"><span>Sistema de origem</span><input value={documentEdit.sourceSystem || ""} onChange={(event) => setDocumentEdit({ ...documentEdit, sourceSystem: event.target.value })} /></label><label className="field"><span>Emissor</span><input value={documentEdit.issuer || ""} onChange={(event) => setDocumentEdit({ ...documentEdit, issuer: event.target.value })} /></label><label className="field"><span>Emissão</span><input type="datetime-local" value={documentEdit.issuedAt || ""} onChange={(event) => setDocumentEdit({ ...documentEdit, issuedAt: event.target.value })} /></label><label className="field"><span>Validade</span><input type="datetime-local" value={documentEdit.validUntil || ""} onChange={(event) => setDocumentEdit({ ...documentEdit, validUntil: event.target.value })} /></label><label className="field full"><span>Caminho do arquivo</span><input value={documentEdit.filePath || ""} onChange={(event) => setDocumentEdit({ ...documentEdit, filePath: event.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={2} value={documentEdit.notes || ""} onChange={(event) => setDocumentEdit({ ...documentEdit, notes: event.target.value })} /></label><label className="field full"><span>Motivo da edição</span><textarea required rows={3} value={documentEdit.reason} onChange={(event) => setDocumentEdit({ ...documentEdit, reason: event.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setDocumentEdit(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !documentEdit.reason.trim() || !documentEdit.referenceNumber.trim()}>Salvar documento</button></div></form>}
    </Dialog>

    <ReasonDialog open={Boolean(reopenTarget)} title="Reabrir transporte temporário?" description={reopenTarget ? `${reopenTarget.colonyCode} · ${reopenTarget.destination || "Transporte"}` : ""} confirmLabel="Reabrir transporte" consequence="O retorno registrado será preservado como revertido e o transporte voltará ao estado aberto. O movimento original permanece intacto." busy={transportBusy} onClose={() => setReopenTarget(null)} onConfirm={async (reason) => { if (!reopenTarget) return false; setTransportBusy(true); setTransportFeedback(null); try { await reopenTransport({ movementId: reopenTarget.id, reason }); setTransportFeedback({ kind: "success", text: "Retorno reaberto com auditoria preservada." }); await reloadMovements(); return true; } catch (error) { setTransportFeedback({ kind: "error", text: publicError(error, "Não foi possível reabrir o transporte.") }); return false; } finally { setTransportBusy(false); } }} />
    <ReasonDialog open={Boolean(movementAction)} title={movementAction?.mode === "reverse" ? "Reverter transferência?" : "Anular transporte?"} description={movementAction ? `${movementAction.item.colonyCode} · ${formatDateTimeBr(movementAction.item.movedAt)}` : ""} confirmLabel={movementAction?.mode === "reverse" ? "Reverter transferência" : "Anular transporte"} consequence={movementAction?.mode === "reverse" ? "A reversão tenta restaurar meliponário, situação e caixa anteriores. Qualquer consequência posterior incompatível bloqueia toda a operação." : "Somente um transporte ainda aberto pode ser anulado. O registro continuará auditável, mas deixará de representar um fato operacional válido."} danger busy={busy} onClose={() => setMovementAction(null)} onConfirm={async (reason) => { if (!movementAction) return false; const payload = { id: movementAction.item.id, reason }; const ok = movementAction.mode === "reverse" ? await onReverseMovement(payload) : await onVoidTransport(payload); if (ok) await reloadMovements(); return ok; }} />
    <ReasonDialog open={Boolean(documentVoid)} title="Invalidar documento?" description={documentVoid ? `${documentLabel(documentVoid.documentType)} · ${documentVoid.referenceNumber}` : ""} confirmLabel="Invalidar documento" consequence="O vínculo e a referência permanecerão preservados na auditoria, mas o documento deixará de ser considerado válido." danger busy={busy} onClose={() => setDocumentVoid(null)} onConfirm={async (reason) => { if (!documentVoid) return false; const ok = await onVoidDocument({ id: documentVoid.id, reason }); if (ok) await reloadDocuments(); return ok; }} />
  </div>;
}

function normalizeDateTime(value?: string) { if (!value) return undefined; const normalized = value.replace("T", " "); return normalized.length === 16 ? `${normalized}:00` : normalized; }
function toInputDateTime(value: string) { return value.replace(" ", "T").slice(0, 16); }
function movementLabel(value: string) { return value === "internal_transfer" ? "Transferência interna" : value === "external_transfer" ? "Transferência externa" : "Transporte temporário"; }
function documentLabel(value: string) { const labels: Record<string,string> = { gta:"GTA", authorization:"Autorização", invoice:"Nota fiscal", receipt:"Recibo", declaration:"Declaração", protocol:"Protocolo", certificate:"Certificado", other:"Outro" }; return labels[value] || value; }
function documentTypeOptions() { return <><option value="gta">GTA</option><option value="authorization">Autorização</option><option value="invoice">Nota fiscal</option><option value="receipt">Recibo</option><option value="declaration">Declaração</option><option value="protocol">Protocolo</option><option value="certificate">Certificado</option><option value="other">Outro</option></>; }
