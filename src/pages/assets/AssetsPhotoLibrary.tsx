import { InspectionPhotoPreview } from "../../components/InspectionPhotoPreview";
import { RecordActions } from "../../components/RecordActions";
import { formatDateTimeBr } from "../../lib/presentation";
import type { Colony, Inspection, InspectionPhoto } from "../../types";
import { formatBytes } from "./presentation";

type Feedback = { kind: "success" | "error"; text: string } | null;

type Props = {
  colonies: Colony[];
  selectedColonyId: string;
  inspections: Inspection[];
  photos: InspectionPhoto[];
  loading: boolean;
  busy: boolean;
  feedback: Feedback;
  onSelectColony: (colonyId: string) => void;
  onOpen: (photoId: string) => void;
  onReveal: (photoId: string) => void;
  onRemove: (photoId: string) => void;
};

export function AssetsPhotoLibrary({ colonies, selectedColonyId, inspections, photos, loading, busy, feedback, onSelectColony, onOpen, onReveal, onRemove }: Props) {
  return <section className="panel wide-list photo-library">
    <div className="panel-heading"><h2>Fotos de inspeção</h2><p>Prévias são carregadas apenas quando entram na área visível e têm tamanho limitado. O arquivo original gerenciado abre pelo sistema.</p></div>
    <label className="field"><span>Colônia</span><select value={selectedColonyId} onChange={(event) => onSelectColony(event.target.value)}><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code}</option>)}</select></label>
    {feedback && <div className={`feedback-banner ${feedback.kind} section-gap`} role={feedback.kind === "error" ? "alert" : "status"}>{feedback.text}</div>}
    {!selectedColonyId ? <div className="empty-list section-gap">Selecione uma colônia para consultar as fotos.</div> : loading ? <div className="empty-list section-gap" role="status">Carregando fotos...</div> : photos.length === 0 ? <div className="empty-list section-gap">Nenhuma foto registrada para esta colônia.</div> : <div className="record-list section-gap">{photos.map((photo) => {
      const inspection = inspections.find((item) => item.id === photo.inspectionId);
      return <article className="record-card photo-record-card" key={photo.id}>
        <InspectionPhotoPreview photoId={photo.id} alt={`Prévia de ${photo.originalName}`} />
        <div className="photo-record-content">
          <div className="record-title-row"><div><strong>{photo.originalName}</strong><span>{formatDateTimeBr(photo.capturedAt)} · {formatBytes(photo.byteSize)}</span></div><RecordActions busy={busy} onOpen={() => onOpen(photo.id)} secondary={[{ label: "Mostrar no local", onClick: () => onReveal(photo.id) }, { label: "Remover…", onClick: () => onRemove(photo.id), danger: true }]} /></div>
          <dl><div><dt>Formato</dt><dd>{photo.mimeType}</dd></div><div><dt>Inspeção</dt><dd>{inspection ? `${formatDateTimeBr(inspection.inspectedAt)}${inspection.boxCode ? ` · ${inspection.boxCode}` : ""}` : "Registro associado"}</dd></div></dl>
          {photo.notes && <p>{photo.notes}</p>}
        </div>
      </article>;
    })}</div>}
  </section>;
}
