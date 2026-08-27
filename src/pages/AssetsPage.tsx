import { useEffect, useState, type FormEvent } from "react";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { Dialog } from "../components/Dialog";
import { PageToolbar } from "../components/PageToolbar";
import { listBoxMaintenance, listColonyInspections, listColonyPhotos } from "../lib/api";
import type { BoxMaintenance, Colony, CreateBoxMaintenanceInput, HiveBox, ImportInspectionPhotoInput, Inspection, InspectionPhoto } from "../types";

type AssetsPageProps = { colonies: Colony[]; boxes: HiveBox[]; busy: boolean; onImportPhoto: (input: ImportInspectionPhotoInput) => Promise<boolean>; onDeletePhoto: (photoId: string) => Promise<boolean>; onCreateMaintenance: (input: CreateBoxMaintenanceInput) => Promise<boolean>; };
const photoInitial: ImportInspectionPhotoInput = { inspectionId: "", sourcePath: "", capturedAt: "", notes: "" };
const maintenanceInitial: CreateBoxMaintenanceInput = { boxId: "", maintainedAt: "", maintenanceType: "inspection", description: "", performedBy: "", nextMaintenanceAt: "" };
const maintenanceTypes = [["cleaning", "Limpeza"], ["repair", "Reparo"], ["painting", "Pintura"], ["waterproofing", "Impermeabilização"], ["roof", "Cobertura"], ["entrance", "Entrada"], ["internal_structure", "Estrutura interna"], ["inspection", "Revisão da caixa"], ["other", "Outro"]] as const;

export function AssetsPage({ colonies, boxes, busy, onImportPhoto, onDeletePhoto, onCreateMaintenance }: AssetsPageProps) {
  const [selectedColonyId, setSelectedColonyId] = useState("");
  const [selectedBoxId, setSelectedBoxId] = useState("");
  const [photoForm, setPhotoForm] = useState<ImportInspectionPhotoInput>(photoInitial);
  const [inspections, setInspections] = useState<Inspection[]>([]);
  const [photos, setPhotos] = useState<InspectionPhoto[]>([]);
  const [photoLoading, setPhotoLoading] = useState(false);
  const [maintenanceForm, setMaintenanceForm] = useState<CreateBoxMaintenanceInput>(maintenanceInitial);
  const [costValue, setCostValue] = useState("");
  const [maintenance, setMaintenance] = useState<BoxMaintenance[]>([]);
  const [maintenanceLoading, setMaintenanceLoading] = useState(false);
  const [photoDialog, setPhotoDialog] = useState(false);
  const [maintenanceDialog, setMaintenanceDialog] = useState(false);
  const [deletePhotoId, setDeletePhotoId] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    if (!selectedColonyId) { setInspections([]); setPhotos([]); return; }
    setPhotoLoading(true);
    Promise.all([listColonyInspections(selectedColonyId), listColonyPhotos(selectedColonyId)]).then(([inspectionItems, photoItems]) => { if (!cancelled) { setInspections(inspectionItems); setPhotos(photoItems); } }).finally(() => { if (!cancelled) setPhotoLoading(false); });
    return () => { cancelled = true; };
  }, [selectedColonyId]);

  useEffect(() => {
    let cancelled = false;
    if (!selectedBoxId) { setMaintenance([]); return; }
    setMaintenanceLoading(true);
    listBoxMaintenance(selectedBoxId).then((items) => { if (!cancelled) setMaintenance(items); }).finally(() => { if (!cancelled) setMaintenanceLoading(false); });
    return () => { cancelled = true; };
  }, [selectedBoxId]);

  async function reloadPhotos() { if (selectedColonyId) setPhotos(await listColonyPhotos(selectedColonyId)); }
  async function reloadMaintenance(boxId = selectedBoxId) { if (boxId) setMaintenance(await listBoxMaintenance(boxId)); }
  function openPhoto() { const firstInspection = inspections[0]?.id || ""; setPhotoForm({ ...photoInitial, inspectionId: firstInspection }); setPhotoDialog(true); }
  function openMaintenance() { setMaintenanceForm({ ...maintenanceInitial, boxId: selectedBoxId }); setCostValue(""); setMaintenanceDialog(true); }

  async function submitPhoto(event: FormEvent) { event.preventDefault(); const input = { ...photoForm, capturedAt: normalizeDateTime(photoForm.capturedAt) }; if (await onImportPhoto(input)) { setPhotoDialog(false); setPhotoForm(photoInitial); await reloadPhotos(); } }
  async function confirmRemovePhoto() { if (!deletePhotoId) return; if (await onDeletePhoto(deletePhotoId)) { setDeletePhotoId(null); await reloadPhotos(); } }
  async function submitMaintenance(event: FormEvent) { event.preventDefault(); const input: CreateBoxMaintenanceInput = { ...maintenanceForm, maintainedAt: normalizeDateTime(maintenanceForm.maintainedAt), nextMaintenanceAt: normalizeDateTime(maintenanceForm.nextMaintenanceAt), cost: costValue.trim() ? Number(costValue) : undefined }; if (await onCreateMaintenance(input)) { const boxId = maintenanceForm.boxId; setSelectedBoxId(boxId); setMaintenanceDialog(false); setMaintenanceForm(maintenanceInitial); setCostValue(""); await reloadMaintenance(boxId); } }

  return <div className="page-stack">
    <PageToolbar title="Manutenção" description="Histórico físico das caixas. Fotos continuam acessíveis como contexto das inspeções." count={`${maintenance.length} manutenções · ${photos.length} fotos`} primaryAction={{ label: "Nova manutenção", onClick: openMaintenance, disabled: busy || boxes.length === 0 }}><button className="button-secondary" type="button" onClick={openPhoto} disabled={busy || !selectedColonyId || inspections.length === 0}>Importar foto de inspeção</button></PageToolbar>
    <div className="content-grid">
      <section className="panel wide-list"><div className="panel-heading"><h2>Histórico da caixa</h2><p>Manutenção acompanha a caixa física, mesmo quando ela estava vazia.</p></div><label className="field"><span>Caixa</span><select value={selectedBoxId} onChange={(e) => setSelectedBoxId(e.target.value)}><option value="">Selecione...</option>{boxes.map((box) => <option value={box.id} key={box.id}>{box.code} {box.currentColonyCode ? `· ${box.currentColonyCode}` : "· vazia"}</option>)}</select></label>{!selectedBoxId ? <div className="empty-list section-gap">Selecione uma caixa para consultar o histórico.</div> : maintenanceLoading ? <div className="empty-list section-gap">Carregando manutenções...</div> : maintenance.length === 0 ? <div className="empty-list section-gap">Nenhuma manutenção registrada.</div> : <div className="table-wrap section-gap"><table className="data-table"><thead><tr><th>Data</th><th>Tipo</th><th>Colônia</th><th>Responsável</th><th>Custo</th><th>Próxima</th></tr></thead><tbody>{maintenance.map((item) => <tr key={item.id}><td><strong>{formatDateTime(item.maintainedAt)}</strong></td><td>{maintenanceLabel(item.maintenanceType)}</td><td>{item.colonyCode || "Caixa vazia"}</td><td>{item.performedBy || "—"}</td><td>{item.cost != null ? `R$ ${item.cost.toFixed(2)}` : "—"}</td><td>{item.nextMaintenanceAt ? formatDateTime(item.nextMaintenanceAt) : "—"}</td></tr>)}</tbody></table></div>}</section>
      <section className="panel wide-list"><div className="panel-heading"><h2>Fotos de inspeção</h2><p>Acesso transitório enquanto fotos migram para as fichas completas das entidades.</p></div><label className="field"><span>Colônia</span><select value={selectedColonyId} onChange={(e) => setSelectedColonyId(e.target.value)}><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code}</option>)}</select></label>{!selectedColonyId ? <div className="empty-list section-gap">Selecione uma colônia para consultar as fotos.</div> : photoLoading ? <div className="empty-list section-gap">Carregando fotos...</div> : photos.length === 0 ? <div className="empty-list section-gap">Nenhuma foto registrada para esta colônia.</div> : <div className="record-list section-gap">{photos.map((photo) => <article className="record-card" key={photo.id}><div className="record-title-row"><div><strong>{photo.originalName}</strong><span>{formatDateTime(photo.capturedAt)} · {formatBytes(photo.byteSize)}</span></div><button type="button" className="button-secondary" onClick={() => setDeletePhotoId(photo.id)} disabled={busy}>Remover</button></div><dl><div><dt>Formato</dt><dd>{photo.mimeType}</dd></div><div><dt>Inspeção</dt><dd>{photo.inspectionId.slice(0, 8)}…</dd></div></dl>{photo.notes && <p>{photo.notes}</p>}</article>)}</div>}</section>
    </div>

    <Dialog open={maintenanceDialog} onClose={() => !busy && setMaintenanceDialog(false)} title="Nova manutenção" description="O contexto da colônia ocupante é resolvido pela data informada." size="large"><form className="form-grid" onSubmit={submitMaintenance}><label className="field full"><span>Caixa</span><select autoFocus required value={maintenanceForm.boxId} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, boxId: e.target.value })}><option value="">Selecione...</option>{boxes.map((box) => <option value={box.id} key={box.id}>{box.code} {box.currentColonyCode ? `· ${box.currentColonyCode}` : "· vazia"}</option>)}</select></label><label className="field"><span>Tipo</span><select value={maintenanceForm.maintenanceType} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, maintenanceType: e.target.value })}>{maintenanceTypes.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label><label className="field"><span>Data e hora</span><input type="datetime-local" value={maintenanceForm.maintainedAt} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, maintainedAt: e.target.value })} /></label><label className="field full"><span>Descrição</span><textarea rows={3} value={maintenanceForm.description} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, description: e.target.value })} /></label><label className="field"><span>Responsável</span><input value={maintenanceForm.performedBy} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, performedBy: e.target.value })} /></label><label className="field"><span>Custo opcional</span><input type="number" min="0" step="0.01" value={costValue} onChange={(e) => setCostValue(e.target.value)} /></label><label className="field full"><span>Próxima manutenção</span><input type="datetime-local" value={maintenanceForm.nextMaintenanceAt} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, nextMaintenanceAt: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setMaintenanceDialog(false)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !maintenanceForm.boxId}>{busy ? "Salvando..." : "Registrar manutenção"}</button></div></form></Dialog>

    <Dialog open={photoDialog} onClose={() => !busy && setPhotoDialog(false)} title="Importar foto de inspeção" description="O backend copia o arquivo para o armazenamento gerenciado; o SQLite guarda os metadados." size="medium"><form className="form-grid" onSubmit={submitPhoto}><label className="field full"><span>Inspeção</span><select autoFocus required value={photoForm.inspectionId} onChange={(e) => setPhotoForm({ ...photoForm, inspectionId: e.target.value })}><option value="">Selecione...</option>{inspections.map((inspection) => <option value={inspection.id} key={inspection.id}>{formatDateTime(inspection.inspectedAt)} {inspection.boxCode ? `· ${inspection.boxCode}` : "· sem caixa"}</option>)}</select></label><label className="field full"><span>Caminho local da foto</span><input value={photoForm.sourcePath} onChange={(e) => setPhotoForm({ ...photoForm, sourcePath: e.target.value })} required /></label><label className="field full"><span>Data da captura opcional</span><input type="datetime-local" value={photoForm.capturedAt} onChange={(e) => setPhotoForm({ ...photoForm, capturedAt: e.target.value })} /></label><label className="field full"><span>Observações</span><textarea rows={2} value={photoForm.notes} onChange={(e) => setPhotoForm({ ...photoForm, notes: e.target.value })} /></label><div className="form-actions full"><button className="button-secondary" type="button" onClick={() => setPhotoDialog(false)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy || !photoForm.inspectionId}>{busy ? "Importando..." : "Importar foto"}</button></div></form></Dialog>

    <ConfirmDialog open={deletePhotoId !== null} title="Remover foto da inspeção?" consequence="A foto será removida do armazenamento gerenciado. O restante do histórico da inspeção permanece intacto." confirmLabel="Remover foto" danger busy={busy} onCancel={() => setDeletePhotoId(null)} onConfirm={() => { void confirmRemovePhoto(); }} />
  </div>;
}
function normalizeDateTime(value?: string) { if (!value) return undefined; const normalized = value.replace("T", " "); return normalized.length === 16 ? `${normalized}:00` : normalized; }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function formatBytes(value: number) { if (value < 1024) return `${value} B`; if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`; return `${(value / (1024 * 1024)).toFixed(1)} MB`; }
function maintenanceLabel(value: string) { return maintenanceTypes.find(([key]) => key === value)?.[1] || value; }
