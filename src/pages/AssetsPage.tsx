import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useEffect, useRef, useState, type FormEvent } from "react";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { ReasonDialog } from "../components/ReasonDialog";
import type { RecordStateMap } from "../hooks/useAppData";
import { listBoxMaintenance, listColonyInspections, listColonyPhotos } from "../lib/api";
import { openInspectionPhoto, revealInspectionPhoto } from "../lib/files-api";
import { createLatestRequestController, runLatestRequest } from "../lib/latest-request";
import { formatDateTimeBr, publicError } from "../lib/presentation";
import type { BoxMaintenance, Colony, CorrectMaintenanceInput, CreateBoxMaintenanceInput, HiveBox, ImportInspectionPhotoInput, Inspection, InspectionPhoto, VoidRecordInput } from "../types";
import { AssetsMaintenancePanel } from "./assets/AssetsMaintenancePanel";
import { AssetsPhotoLibrary } from "./assets/AssetsPhotoLibrary";
import { fileNameFromPath, maintenanceLabel, maintenanceTypes, normalizeDateTime, toInputDateTime } from "./assets/presentation";

type Props = {
  colonies: Colony[];
  boxes: HiveBox[];
  busy: boolean;
  recordStateMap: RecordStateMap;
  autoCreate?: boolean;
  onImportPhoto: (input: ImportInspectionPhotoInput) => Promise<boolean>;
  onDeletePhoto: (photoId: string) => Promise<boolean>;
  onCreateMaintenance: (input: CreateBoxMaintenanceInput) => Promise<boolean>;
  onCorrectMaintenance: (input: CorrectMaintenanceInput) => Promise<boolean>;
  onVoidMaintenance: (input: VoidRecordInput) => Promise<boolean>;
};

const photoInitial: ImportInspectionPhotoInput = { inspectionId: "", sourcePath: "", capturedAt: "", notes: "" };
const maintenanceInitial: CreateBoxMaintenanceInput = { boxId: "", maintainedAt: "", maintenanceType: "inspection", description: "", performedBy: "", nextMaintenanceAt: "" };

export function AssetsPage({ colonies, boxes, busy, recordStateMap, autoCreate = false, onImportPhoto, onDeletePhoto, onCreateMaintenance, onCorrectMaintenance, onVoidMaintenance }: Props) {
  const [selectedColonyId, setSelectedColonyId] = useState("");
  const [selectedBoxId, setSelectedBoxId] = useState("");
  const [photoForm, setPhotoForm] = useState<ImportInspectionPhotoInput>(photoInitial);
  const [inspections, setInspections] = useState<Inspection[]>([]);
  const [photos, setPhotos] = useState<InspectionPhoto[]>([]);
  const [photoLoading, setPhotoLoading] = useState(false);
  const [photoFeedback, setPhotoFeedback] = useState<{ kind: "success" | "error"; text: string } | null>(null);
  const [maintenanceForm, setMaintenanceForm] = useState<CreateBoxMaintenanceInput>(maintenanceInitial);
  const [costValue, setCostValue] = useState("");
  const [maintenance, setMaintenance] = useState<BoxMaintenance[]>([]);
  const [maintenanceLoading, setMaintenanceLoading] = useState(false);
  const [maintenanceError, setMaintenanceError] = useState("");
  const [photoDialog, setPhotoDialog] = useState(false);
  const [maintenanceDialog, setMaintenanceDialog] = useState(false);
  const [deletePhotoId, setDeletePhotoId] = useState<string | null>(null);
  const [maintenanceDetail, setMaintenanceDetail] = useState<BoxMaintenance | null>(null);
  const [editMaintenance, setEditMaintenance] = useState<CorrectMaintenanceInput | null>(null);
  const [voidMaintenance, setVoidMaintenance] = useState<BoxMaintenance | null>(null);
  const handledAutoCreate = useRef(false);
  const photoRequests = useRef(createLatestRequestController());
  const maintenanceRequests = useRef(createLatestRequestController());

  useEffect(() => { void reloadPhotoContext(selectedColonyId); }, [selectedColonyId]);
  useEffect(() => { void reloadMaintenance(selectedBoxId); }, [selectedBoxId]);
  useEffect(() => {
    if (!autoCreate) { handledAutoCreate.current = false; return; }
    if (handledAutoCreate.current || boxes.length !== 1) return;
    handledAutoCreate.current = true;
    const boxId = boxes[0].id;
    setSelectedBoxId(boxId);
    setMaintenanceForm({ ...maintenanceInitial, boxId });
    setCostValue("");
    setMaintenanceDialog(true);
  }, [autoCreate, boxes]);

  async function reloadPhotoContext(colonyId = selectedColonyId) {
    if (!colonyId) {
      photoRequests.current.invalidate();
      setInspections([]);
      setPhotos([]);
      setPhotoLoading(false);
      return "stale" as const;
    }
    setInspections([]);
    setPhotos([]);
    setPhotoLoading(true);
    setPhotoFeedback(null);
    return runLatestRequest(
      photoRequests.current,
      () => Promise.all([listColonyInspections(colonyId), listColonyPhotos(colonyId)]),
      {
        onSuccess: ([inspectionItems, photoItems]) => {
          setInspections(inspectionItems);
          setPhotos(photoItems);
        },
        onError: (error) => {
          setPhotoFeedback({ kind: "error", text: publicError(error, "Não foi possível carregar as fotos desta colônia.") });
        },
        onSettled: () => setPhotoLoading(false),
      },
    );
  }

  async function reloadPhotos(colonyId = selectedColonyId) {
    if (!colonyId) {
      photoRequests.current.invalidate();
      setPhotos([]);
      setPhotoLoading(false);
      return "stale" as const;
    }
    setPhotoLoading(true);
    return runLatestRequest(
      photoRequests.current,
      () => listColonyPhotos(colonyId),
      {
        onSuccess: setPhotos,
        onError: (error) => {
          setPhotoFeedback({ kind: "error", text: publicError(error, "A alteração foi salva, mas não foi possível recarregar as fotos desta colônia.") });
        },
        onSettled: () => setPhotoLoading(false),
      },
    );
  }

  async function reloadMaintenance(boxId = selectedBoxId) {
    if (!boxId) {
      maintenanceRequests.current.invalidate();
      setMaintenance([]);
      setMaintenanceLoading(false);
      setMaintenanceError("");
      return "stale" as const;
    }
    setMaintenanceLoading(true);
    setMaintenanceError("");
    return runLatestRequest(
      maintenanceRequests.current,
      () => listBoxMaintenance(boxId),
      {
        onSuccess: setMaintenance,
        onError: (error) => setMaintenanceError(publicError(error, "Não foi possível carregar as manutenções desta caixa.")),
        onSettled: () => setMaintenanceLoading(false),
      },
    );
  }

  function openPhoto() {
    const firstInspection = inspections[0]?.id || "";
    setPhotoFeedback(null);
    setPhotoForm({ ...photoInitial, inspectionId: firstInspection });
    setPhotoDialog(true);
  }

  function openMaintenance() {
    setMaintenanceForm({ ...maintenanceInitial, boxId: selectedBoxId });
    setCostValue("");
    setMaintenanceDialog(true);
  }

  function beginCorrect(item: BoxMaintenance) {
    setEditMaintenance({ id: item.id, reason: "", boxId: item.boxId, maintainedAt: toInputDateTime(item.maintainedAt), maintenanceType: item.maintenanceType, description: item.description || "", performedBy: item.performedBy || "", cost: item.cost ?? undefined, nextMaintenanceAt: item.nextMaintenanceAt ? toInputDateTime(item.nextMaintenanceAt) : "" });
  }

  async function choosePhotoFile() {
    const selected = await openDialog({ multiple: false, directory: false, title: "Selecionar foto da inspeção", filters: [{ name: "Imagens", extensions: ["jpg", "jpeg", "png", "webp"] }] });
    if (typeof selected === "string") setPhotoForm({ ...photoForm, sourcePath: selected });
  }

  async function runPhotoAction(action: "open" | "reveal", photoId: string) {
    setPhotoFeedback(null);
    try {
      if (action === "open") await openInspectionPhoto(photoId);
      else await revealInspectionPhoto(photoId);
    } catch (error) {
      setPhotoFeedback({ kind: "error", text: publicError(error, action === "open" ? "Não foi possível abrir a foto." : "Não foi possível mostrar a foto no local.") });
    }
  }

  async function submitPhoto(event: FormEvent) {
    event.preventDefault();
    const input = { ...photoForm, capturedAt: normalizeDateTime(photoForm.capturedAt) };
    if (await onImportPhoto(input)) {
      setPhotoDialog(false);
      setPhotoForm(photoInitial);
      if (await reloadPhotos() === "success") {
        setPhotoFeedback({ kind: "success", text: "Foto importada para a área gerenciada." });
      }
    }
  }

  async function confirmRemovePhoto() {
    if (!deletePhotoId) return;
    if (await onDeletePhoto(deletePhotoId)) {
      setDeletePhotoId(null);
      if (await reloadPhotos() === "success") {
        setPhotoFeedback({ kind: "success", text: "Foto removida da área gerenciada." });
      }
    }
  }

  async function submitMaintenance(event: FormEvent) {
    event.preventDefault();
    const input: CreateBoxMaintenanceInput = { ...maintenanceForm, maintainedAt: normalizeDateTime(maintenanceForm.maintainedAt), nextMaintenanceAt: normalizeDateTime(maintenanceForm.nextMaintenanceAt), cost: costValue.trim() ? Number(costValue) : undefined };
    if (await onCreateMaintenance(input)) {
      const boxId = maintenanceForm.boxId;
      setSelectedBoxId(boxId);
      setMaintenanceDialog(false);
      setMaintenanceForm(maintenanceInitial);
      setCostValue("");
      await reloadMaintenance(boxId);
    }
  }

  async function submitMaintenanceCorrection(event: FormEvent) {
    event.preventDefault();
    if (!editMaintenance) return;
    const payload: CorrectMaintenanceInput = { ...editMaintenance, maintainedAt: normalizeDateTime(editMaintenance.maintainedAt) || editMaintenance.maintainedAt, nextMaintenanceAt: normalizeDateTime(editMaintenance.nextMaintenanceAt) };
    if (await onCorrectMaintenance(payload)) {
      setSelectedBoxId(payload.boxId);
      setEditMaintenance(null);
      await reloadMaintenance(payload.boxId);
    }
  }

  return <div className="page-stack">
    <PageToolbar title="Manutenção" description="Histórico físico das caixas. Fotos permanecem vinculadas às inspeções e armazenadas na área gerenciada." count={`${maintenance.length} manutenções · ${photos.length} fotos`} primaryAction={{ label: "Nova manutenção", onClick: openMaintenance, disabled: busy || boxes.length === 0 }}>
      <button className="button-secondary" type="button" onClick={openPhoto} disabled={busy || !selectedColonyId || inspections.length === 0}>Importar foto…</button>
    </PageToolbar>

    <div className="content-grid">
      <AssetsMaintenancePanel
        boxes={boxes}
        selectedBoxId={selectedBoxId}
        maintenance={maintenance}
        loading={maintenanceLoading}
        error={maintenanceError}
        busy={busy}
        recordStateMap={recordStateMap}
        onSelectBox={setSelectedBoxId}
        onOpen={setMaintenanceDetail}
        onEdit={beginCorrect}
        onVoid={setVoidMaintenance}
      />
      <AssetsPhotoLibrary
        colonies={colonies}
        selectedColonyId={selectedColonyId}
        inspections={inspections}
        photos={photos}
        loading={photoLoading}
        busy={busy}
        feedback={photoFeedback}
        onSelectColony={setSelectedColonyId}
        onOpen={(photoId) => void runPhotoAction("open", photoId)}
        onReveal={(photoId) => void runPhotoAction("reveal", photoId)}
        onRemove={setDeletePhotoId}
      />
    </div>

    <Dialog open={maintenanceDialog} onClose={() => !busy && setMaintenanceDialog(false)} title="Nova manutenção" description="O contexto da colônia ocupante é resolvido pela data informada." size="large">
      <form className="form-grid" onSubmit={submitMaintenance}><label className="field full"><span>Caixa</span><select autoFocus required value={maintenanceForm.boxId} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, boxId: e.target.value })}><option value="">Selecione...</option>{boxes.map((box) => <option value={box.id} key={box.id}>{box.code} {box.currentColonyCode ? `· ${box.currentColonyCode}` : "· vazia"}</option>)}</select></label><label className="field"><span>Tipo</span><select value={maintenanceForm.maintenanceType} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, maintenanceType: e.target.value })}>{maintenanceTypes.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label><label className="field"><span>Data e hora</span><input type="datetime-local" value={maintenanceForm.maintainedAt} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, maintainedAt: e.target.value })} /></label><label className="field full"><span>Descrição</span><textarea rows={3} value={maintenanceForm.description} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, description: e.target.value })} /></label><label className="field"><span>Responsável</span><input value={maintenanceForm.performedBy} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, performedBy: e.target.value })} /></label><label className="field"><span>Custo opcional</span><input type="number" min="0" step="0.01" value={costValue} onChange={(e) => setCostValue(e.target.value)} /></label><label className="field full"><span>Próxima manutenção</span><input type="datetime-local" value={maintenanceForm.nextMaintenanceAt} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, nextMaintenanceAt: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setMaintenanceDialog(false)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !maintenanceForm.boxId}>{busy ? "Salvando..." : "Registrar manutenção"}</button></div></form>
    </Dialog>

    <Dialog open={Boolean(maintenanceDetail)} onClose={() => setMaintenanceDetail(null)} title="Manutenção da caixa" description={maintenanceDetail ? `${maintenanceDetail.boxCode} · ${formatDateTimeBr(maintenanceDetail.maintainedAt)}` : ""} size="medium">
      {maintenanceDetail && <div className="detail-grid"><div><span>Tipo</span><strong>{maintenanceLabel(maintenanceDetail.maintenanceType)}</strong></div><div><span>Colônia</span><strong>{maintenanceDetail.colonyCode || "Caixa vazia"}</strong></div><div><span>Responsável</span><strong>{maintenanceDetail.performedBy || "—"}</strong></div><div><span>Custo</span><strong>{maintenanceDetail.cost != null ? `R$ ${maintenanceDetail.cost.toFixed(2)}` : "—"}</strong></div><div className="full"><span>Descrição</span><p>{maintenanceDetail.description || "—"}</p></div></div>}
    </Dialog>

    <Dialog open={Boolean(editMaintenance)} onClose={() => !busy && setEditMaintenance(null)} title="Corrigir manutenção" description="A correção recalcula o contexto histórico da caixa e registra antes/depois na auditoria." size="large">
      {editMaintenance && <form className="form-grid" onSubmit={submitMaintenanceCorrection}><label className="field full"><span>Caixa</span><select autoFocus required value={editMaintenance.boxId} onChange={(e) => setEditMaintenance({ ...editMaintenance, boxId: e.target.value })}>{boxes.map((box) => <option value={box.id} key={box.id}>{box.code}</option>)}</select></label><label className="field"><span>Tipo</span><select value={editMaintenance.maintenanceType} onChange={(e) => setEditMaintenance({ ...editMaintenance, maintenanceType: e.target.value })}>{maintenanceTypes.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label><label className="field"><span>Data e hora</span><input required type="datetime-local" value={editMaintenance.maintainedAt} onChange={(e) => setEditMaintenance({ ...editMaintenance, maintainedAt: e.target.value })} /></label><label className="field full"><span>Descrição</span><textarea rows={3} value={editMaintenance.description || ""} onChange={(e) => setEditMaintenance({ ...editMaintenance, description: e.target.value })} /></label><label className="field"><span>Responsável</span><input value={editMaintenance.performedBy || ""} onChange={(e) => setEditMaintenance({ ...editMaintenance, performedBy: e.target.value })} /></label><label className="field"><span>Custo</span><input type="number" min="0" step="0.01" value={editMaintenance.cost ?? ""} onChange={(e) => setEditMaintenance({ ...editMaintenance, cost: e.target.value ? Number(e.target.value) : undefined })} /></label><label className="field full"><span>Próxima manutenção</span><input type="datetime-local" value={editMaintenance.nextMaintenanceAt || ""} onChange={(e) => setEditMaintenance({ ...editMaintenance, nextMaintenanceAt: e.target.value })} /></label><label className="field full"><span>Motivo da correção</span><textarea required rows={3} value={editMaintenance.reason} onChange={(e) => setEditMaintenance({ ...editMaintenance, reason: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setEditMaintenance(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !editMaintenance.reason.trim()}>Salvar correção</button></div></form>}
    </Dialog>

    <Dialog open={photoDialog} onClose={() => !busy && setPhotoDialog(false)} title="Importar foto de inspeção" description="A foto é copiada para o armazenamento gerenciado; o arquivo escolhido no computador permanece intacto." size="medium">
      <form className="form-grid" onSubmit={submitPhoto}><label className="field full"><span>Inspeção</span><select autoFocus required value={photoForm.inspectionId} onChange={(e) => setPhotoForm({ ...photoForm, inspectionId: e.target.value })}><option value="">Selecione...</option>{inspections.map((inspection) => <option value={inspection.id} key={inspection.id}>{formatDateTimeBr(inspection.inspectedAt)} {inspection.boxCode ? `· ${inspection.boxCode}` : "· sem caixa"}</option>)}</select></label><div className="field full"><span>Foto</span><div className="file-picker-row"><button className="button-secondary" type="button" onClick={() => void choosePhotoFile()} disabled={busy}>Selecionar foto…</button><strong>{photoForm.sourcePath ? fileNameFromPath(photoForm.sourcePath) : "Nenhum arquivo selecionado"}</strong></div></div><label className="field full"><span>Data da captura opcional</span><input type="datetime-local" value={photoForm.capturedAt} onChange={(e) => setPhotoForm({ ...photoForm, capturedAt: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={2} value={photoForm.notes} onChange={(e) => setPhotoForm({ ...photoForm, notes: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setPhotoDialog(false)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !photoForm.inspectionId || !photoForm.sourcePath}>{busy ? "Importando..." : "Importar foto"}</button></div></form>
    </Dialog>

    <ReasonDialog open={Boolean(voidMaintenance)} title="Anular manutenção" description={voidMaintenance ? `${voidMaintenance.boxCode} · ${formatDateTimeBr(voidMaintenance.maintainedAt)}` : ""} confirmLabel="Anular registro" consequence="A manutenção permanecerá auditável, mas deixará de representar um fato operacional válido." danger busy={busy} onClose={() => setVoidMaintenance(null)} onConfirm={async (reason) => { if (!voidMaintenance) return false; const ok = await onVoidMaintenance({ id: voidMaintenance.id, reason }); if (ok) await reloadMaintenance(); return ok; }} />
    <ConfirmDialog open={deletePhotoId !== null} title="Remover foto da inspeção?" consequence="A cópia da foto será removida do armazenamento gerenciado. O arquivo original importado e o restante do histórico da inspeção permanecem intactos." confirmLabel="Remover foto" danger busy={busy} onCancel={() => setDeletePhotoId(null)} onConfirm={() => { void confirmRemovePhoto(); }} />
  </div>;
}
