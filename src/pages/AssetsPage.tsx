import { useEffect, useState, type FormEvent } from "react";
import { listBoxMaintenance, listColonyInspections, listColonyPhotos } from "../lib/api";
import type { BoxMaintenance, Colony, CreateBoxMaintenanceInput, HiveBox, ImportInspectionPhotoInput, Inspection, InspectionPhoto } from "../types";

type AssetsPageProps = {
  colonies: Colony[];
  boxes: HiveBox[];
  busy: boolean;
  onImportPhoto: (input: ImportInspectionPhotoInput) => Promise<boolean>;
  onDeletePhoto: (photoId: string) => Promise<boolean>;
  onCreateMaintenance: (input: CreateBoxMaintenanceInput) => Promise<boolean>;
};

const photoInitial: ImportInspectionPhotoInput = { inspectionId: "", sourcePath: "", capturedAt: "", notes: "" };
const maintenanceInitial: CreateBoxMaintenanceInput = { boxId: "", maintainedAt: "", maintenanceType: "inspection", description: "", performedBy: "", nextMaintenanceAt: "" };

export function AssetsPage({ colonies, boxes, busy, onImportPhoto, onDeletePhoto, onCreateMaintenance }: AssetsPageProps) {
  const [selectedColonyId, setSelectedColonyId] = useState("");
  const [photoForm, setPhotoForm] = useState<ImportInspectionPhotoInput>(photoInitial);
  const [inspections, setInspections] = useState<Inspection[]>([]);
  const [photos, setPhotos] = useState<InspectionPhoto[]>([]);
  const [photoLoading, setPhotoLoading] = useState(false);
  const [maintenanceForm, setMaintenanceForm] = useState<CreateBoxMaintenanceInput>(maintenanceInitial);
  const [costValue, setCostValue] = useState("");
  const [maintenance, setMaintenance] = useState<BoxMaintenance[]>([]);
  const [maintenanceLoading, setMaintenanceLoading] = useState(false);

  useEffect(() => {
    let cancelled = false;
    if (!selectedColonyId) {
      setInspections([]);
      setPhotos([]);
      setPhotoForm(photoInitial);
      return;
    }
    setPhotoLoading(true);
    Promise.all([listColonyInspections(selectedColonyId), listColonyPhotos(selectedColonyId)])
      .then(([inspectionItems, photoItems]) => {
        if (cancelled) return;
        setInspections(inspectionItems);
        setPhotos(photoItems);
        setPhotoForm((current) => ({ ...current, inspectionId: inspectionItems.some((item) => item.id === current.inspectionId) ? current.inspectionId : inspectionItems[0]?.id || "" }));
      })
      .finally(() => { if (!cancelled) setPhotoLoading(false); });
    return () => { cancelled = true; };
  }, [selectedColonyId]);

  useEffect(() => {
    let cancelled = false;
    if (!maintenanceForm.boxId) { setMaintenance([]); return; }
    setMaintenanceLoading(true);
    listBoxMaintenance(maintenanceForm.boxId)
      .then((items) => { if (!cancelled) setMaintenance(items); })
      .finally(() => { if (!cancelled) setMaintenanceLoading(false); });
    return () => { cancelled = true; };
  }, [maintenanceForm.boxId]);

  async function reloadPhotos() {
    if (selectedColonyId) setPhotos(await listColonyPhotos(selectedColonyId));
  }

  async function reloadMaintenance(boxId = maintenanceForm.boxId) {
    if (boxId) setMaintenance(await listBoxMaintenance(boxId));
  }

  async function submitPhoto(event: FormEvent) {
    event.preventDefault();
    const input: ImportInspectionPhotoInput = { ...photoForm, capturedAt: normalizeDateTime(photoForm.capturedAt) };
    if (await onImportPhoto(input)) {
      const inspectionId = photoForm.inspectionId;
      setPhotoForm({ ...photoInitial, inspectionId });
      await reloadPhotos();
    }
  }

  async function removePhoto(photoId: string) {
    if (!window.confirm("Remover esta foto do armazenamento gerenciado?")) return;
    if (await onDeletePhoto(photoId)) await reloadPhotos();
  }

  async function submitMaintenance(event: FormEvent) {
    event.preventDefault();
    const input: CreateBoxMaintenanceInput = {
      ...maintenanceForm,
      maintainedAt: normalizeDateTime(maintenanceForm.maintainedAt),
      nextMaintenanceAt: normalizeDateTime(maintenanceForm.nextMaintenanceAt),
      cost: costValue.trim() ? Number(costValue) : undefined,
    };
    if (await onCreateMaintenance(input)) {
      const boxId = maintenanceForm.boxId;
      setMaintenanceForm({ ...maintenanceInitial, boxId });
      setCostValue("");
      await reloadMaintenance(boxId);
    }
  }

  return (
    <div className="page-stack">
      <section className="page-heading">
        <div><span className="eyebrow">Acervo e estrutura</span><h1>Fotos e manutenção</h1><p>Fotos documentam inspeções. Manutenções documentam a caixa física, inclusive quando ela estava vazia.</p></div>
        <span className="count-pill">{photos.length} foto{photos.length === 1 ? "" : "s"} · {maintenance.length} manutenção{maintenance.length === 1 ? "" : "ões"}</span>
      </section>

      <div className="content-grid">
        <section className="panel form-panel">
          <div className="panel-heading"><h2>Foto de inspeção</h2><p>JPG, PNG ou WebP. O backend copia o arquivo para o diretório gerenciado e guarda apenas metadados no SQLite.</p></div>
          <form className="form-grid" onSubmit={submitPhoto}>
            <label className="field full"><span>Colônia</span><select value={selectedColonyId} onChange={(e) => setSelectedColonyId(e.target.value)} required><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code}</option>)}</select></label>
            <label className="field full"><span>Inspeção</span><select value={photoForm.inspectionId} onChange={(e) => setPhotoForm({ ...photoForm, inspectionId: e.target.value })} required disabled={!selectedColonyId || inspections.length === 0}><option value="">Selecione...</option>{inspections.map((inspection) => <option value={inspection.id} key={inspection.id}>{formatDateTime(inspection.inspectedAt)} {inspection.boxCode ? `· ${inspection.boxCode}` : "· sem caixa"}</option>)}</select></label>
            {selectedColonyId && inspections.length === 0 && !photoLoading && <div className="inline-notice field full">Esta colônia ainda não possui inspeções. Registre uma inspeção antes de anexar fotos.</div>}
            <label className="field full"><span>Caminho local da foto</span><input value={photoForm.sourcePath} onChange={(e) => setPhotoForm({ ...photoForm, sourcePath: e.target.value })} placeholder="Ex.: /home/usuario/fotos/IMG_0001.jpg" required /></label>
            <label className="field full"><span>Data da captura opcional</span><input type="datetime-local" value={photoForm.capturedAt} onChange={(e) => setPhotoForm({ ...photoForm, capturedAt: e.target.value })} /></label>
            <label className="field full"><span>Observações</span><textarea rows={2} value={photoForm.notes} onChange={(e) => setPhotoForm({ ...photoForm, notes: e.target.value })} /></label>
            <div className="form-actions full"><button type="submit" disabled={busy || !photoForm.inspectionId}>{busy ? "Importando..." : "Importar foto"}</button></div>
          </form>
        </section>

        <section className="panel list-panel">
          <div className="panel-heading"><h2>Acervo da colônia</h2><p>Os nomes originais são preservados nos metadados; o arquivo interno usa UUID.</p></div>
          {!selectedColonyId ? <div className="empty-list">Selecione uma colônia para consultar as fotos.</div> : photoLoading ? <div className="empty-list">Carregando fotos...</div> : photos.length === 0 ? <div className="empty-list">Nenhuma foto registrada para esta colônia.</div> : <div className="record-list">{photos.map((photo) => <article className="record-card" key={photo.id}><div className="record-title-row"><div><strong>{photo.originalName}</strong><span>{formatDateTime(photo.capturedAt)} · {formatBytes(photo.byteSize)}</span></div><button type="button" className="button-secondary" onClick={() => void removePhoto(photo.id)} disabled={busy}>Remover</button></div><dl><div><dt>Formato</dt><dd>{photo.mimeType}</dd></div><div><dt>Inspeção</dt><dd>{photo.inspectionId.slice(0, 8)}…</dd></div></dl>{photo.notes && <p>{photo.notes}</p>}</article>)}</div>}
        </section>
      </div>

      <div className="content-grid">
        <section className="panel form-panel">
          <div className="panel-heading"><h2>Manutenção da caixa</h2><p>O contexto da colônia ocupante é resolvido pela data informada; uma caixa vazia permanece sem colônia associada.</p></div>
          <form className="form-grid" onSubmit={submitMaintenance}>
            <label className="field full"><span>Caixa</span><select required value={maintenanceForm.boxId} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, boxId: e.target.value })}><option value="">Selecione...</option>{boxes.map((box) => <option value={box.id} key={box.id}>{box.code} {box.currentColonyCode ? `· ${box.currentColonyCode}` : "· vazia"}</option>)}</select></label>
            <label className="field"><span>Tipo</span><select value={maintenanceForm.maintenanceType} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, maintenanceType: e.target.value })}>{maintenanceTypes.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label>
            <label className="field"><span>Data e hora</span><input type="datetime-local" value={maintenanceForm.maintainedAt} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, maintainedAt: e.target.value })} /></label>
            <label className="field full"><span>Descrição</span><textarea rows={3} value={maintenanceForm.description} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, description: e.target.value })} /></label>
            <label className="field"><span>Responsável</span><input value={maintenanceForm.performedBy} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, performedBy: e.target.value })} /></label>
            <label className="field"><span>Custo opcional</span><input type="number" min="0" step="0.01" value={costValue} onChange={(e) => setCostValue(e.target.value)} /></label>
            <label className="field full"><span>Próxima manutenção</span><input type="datetime-local" value={maintenanceForm.nextMaintenanceAt} onChange={(e) => setMaintenanceForm({ ...maintenanceForm, nextMaintenanceAt: e.target.value })} /></label>
            <div className="form-actions full"><button type="submit" disabled={busy || !maintenanceForm.boxId}>{busy ? "Salvando..." : "Registrar manutenção"}</button></div>
          </form>
        </section>

        <section className="panel list-panel">
          <div className="panel-heading"><h2>Histórico da caixa</h2><p>Manutenção segue a identidade física da caixa, independentemente das colônias que passaram por ela.</p></div>
          {!maintenanceForm.boxId ? <div className="empty-list">Selecione uma caixa para consultar o histórico.</div> : maintenanceLoading ? <div className="empty-list">Carregando manutenções...</div> : maintenance.length === 0 ? <div className="empty-list">Nenhuma manutenção registrada.</div> : <div className="record-list">{maintenance.map((item) => <article className="record-card" key={item.id}><div className="record-title-row"><div><strong>{maintenanceLabel(item.maintenanceType)}</strong><span>{formatDateTime(item.maintainedAt)} · {item.colonyCode ? `ocupada por ${item.colonyCode}` : "caixa vazia"}</span></div>{item.cost != null && <span className="badge">R$ {item.cost.toFixed(2)}</span>}</div>{item.description && <p>{item.description}</p>}<dl><div><dt>Responsável</dt><dd>{item.performedBy || "Não informado"}</dd></div><div><dt>Próxima</dt><dd>{item.nextMaintenanceAt ? formatDateTime(item.nextMaintenanceAt) : "Sem agendamento"}</dd></div></dl></article>)}</div>}
        </section>
      </div>
    </div>
  );
}

const maintenanceTypes = [["cleaning", "Limpeza"], ["repair", "Reparo"], ["painting", "Pintura"], ["waterproofing", "Impermeabilização"], ["roof", "Cobertura"], ["entrance", "Entrada"], ["internal_structure", "Estrutura interna"], ["inspection", "Revisão da caixa"], ["other", "Outro"]] as const;
function normalizeDateTime(value?: string) { if (!value) return undefined; const normalized = value.replace("T", " "); return normalized.length === 16 ? `${normalized}:00` : normalized; }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function formatBytes(value: number) { if (value < 1024) return `${value} B`; if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`; return `${(value / (1024 * 1024)).toFixed(1)} MB`; }
function maintenanceLabel(value: string) { return maintenanceTypes.find(([key]) => key === value)?.[1] || value; }
