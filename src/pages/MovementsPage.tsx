import { useEffect, useMemo, useState, type FormEvent } from "react";
import { listColonyMovements, listMovementDocuments } from "../lib/api";
import type { Colony, ColonyMovement, CreateMovementDocumentInput, CreateMovementInput, HiveBox, Meliponary, MovementDocument } from "../types";

type Props = { colonies: Colony[]; meliponaries: Meliponary[]; boxes: HiveBox[]; busy: boolean; onCreateMovement: (input: CreateMovementInput) => Promise<boolean>; onCreateDocument: (input: CreateMovementDocumentInput) => Promise<boolean>; };
const movementInitial: CreateMovementInput = { colonyId: "", movementType: "transport", movedAt: "", toMeliponaryId: "", toBoxId: "", destination: "", notes: "" };
const documentInitial: CreateMovementDocumentInput = { movementId: "", documentType: "gta", referenceNumber: "", sourceSystem: "", issuer: "", issuedAt: "", validUntil: "", filePath: "", notes: "" };

export function MovementsPage({ colonies, meliponaries, boxes, busy, onCreateMovement, onCreateDocument }: Props) {
  const [movementForm, setMovementForm] = useState<CreateMovementInput>(movementInitial);
  const [documentForm, setDocumentForm] = useState<CreateMovementDocumentInput>(documentInitial);
  const [movements, setMovements] = useState<ColonyMovement[]>([]);
  const [documents, setDocuments] = useState<MovementDocument[]>([]);
  const [loading, setLoading] = useState(false);
  const selectedColony = colonies.find((c) => c.id === movementForm.colonyId);
  const movable = selectedColony ? !["lost", "inactive", "transferred"].includes(selectedColony.status) : false;
  const targetBoxes = useMemo(() => boxes.filter((box) => box.meliponaryId === movementForm.toMeliponaryId && box.status === "active" && !box.currentColonyCode), [boxes, movementForm.toMeliponaryId]);

  useEffect(() => {
    let cancelled = false;
    if (!movementForm.colonyId) { setMovements([]); setDocumentForm(documentInitial); setDocuments([]); return; }
    setLoading(true);
    listColonyMovements(movementForm.colonyId).then((items) => { if (!cancelled) setMovements(items); }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [movementForm.colonyId]);

  useEffect(() => {
    let cancelled = false;
    if (!documentForm.movementId) { setDocuments([]); return; }
    listMovementDocuments(documentForm.movementId).then((items) => { if (!cancelled) setDocuments(items); });
    return () => { cancelled = true; };
  }, [documentForm.movementId]);

  async function reloadMovements(colonyId = movementForm.colonyId) { if (colonyId) setMovements(await listColonyMovements(colonyId)); }
  async function reloadDocuments(movementId = documentForm.movementId) { if (movementId) setDocuments(await listMovementDocuments(movementId)); }

  async function submitMovement(event: FormEvent) {
    event.preventDefault();
    const input: CreateMovementInput = { ...movementForm, movedAt: normalizeDateTime(movementForm.movedAt), toMeliponaryId: movementForm.movementType === "internal_transfer" ? movementForm.toMeliponaryId : undefined, toBoxId: movementForm.movementType === "internal_transfer" ? movementForm.toBoxId : undefined, destination: movementForm.movementType === "internal_transfer" ? undefined : movementForm.destination, documentReference: undefined };
    if (await onCreateMovement(input)) { const colonyId = movementForm.colonyId; setMovementForm({ ...movementInitial, colonyId }); await reloadMovements(colonyId); }
  }

  async function submitDocument(event: FormEvent) {
    event.preventDefault();
    const input: CreateMovementDocumentInput = { ...documentForm, issuedAt: normalizeDateTime(documentForm.issuedAt), validUntil: normalizeDateTime(documentForm.validUntil) };
    if (await onCreateDocument(input)) { const movementId = documentForm.movementId; setDocumentForm({ ...documentInitial, movementId }); await reloadDocuments(movementId); }
  }

  return <div className="page-stack">
    <section className="page-heading"><div><span className="eyebrow">Rastreabilidade</span><h1>Movimentações e documentos</h1><p>Registre deslocamentos do plantel e depois vincule os documentos que comprovam ou contextualizam cada movimentação.</p></div><span className="count-pill">{movements.length} movimentação{movements.length === 1 ? "" : "ões"}</span></section>
    <div className="content-grid">
      <section className="panel form-panel"><div className="panel-heading"><h2>Nova movimentação</h2><p>Transferência interna altera a localização cadastrada; transporte não altera o estado atual.</p></div>
        <form className="form-grid" onSubmit={submitMovement}>
          <label className="field full"><span>Colônia</span><select required value={movementForm.colonyId} onChange={(e) => setMovementForm({ ...movementForm, colonyId: e.target.value, toMeliponaryId: "", toBoxId: "" })}><option value="">Selecione...</option>{colonies.map((c) => <option value={c.id} key={c.id}>{c.code} · {c.status}</option>)}</select></label>
          {selectedColony && !movable && <div className="inline-notice field full">Esta colônia pode ter o histórico consultado, mas não está disponível para nova movimentação.</div>}
          <label className="field"><span>Tipo</span><select value={movementForm.movementType} onChange={(e) => setMovementForm({ ...movementForm, movementType: e.target.value, toMeliponaryId: "", toBoxId: "", destination: "" })}><option value="transport">Transporte temporário</option><option value="internal_transfer">Transferência interna</option><option value="external_transfer">Transferência externa</option></select></label>
          <label className="field"><span>Data e hora</span><input type="datetime-local" value={movementForm.movedAt} onChange={(e) => setMovementForm({ ...movementForm, movedAt: e.target.value })} /></label>
          {movementForm.movementType === "internal_transfer" ? <>
            <label className="field full"><span>Meliponário de destino</span><select required value={movementForm.toMeliponaryId} onChange={(e) => setMovementForm({ ...movementForm, toMeliponaryId: e.target.value, toBoxId: "" })}><option value="">Selecione...</option>{meliponaries.filter((m) => m.id !== selectedColony?.meliponaryId).map((m) => <option value={m.id} key={m.id}>{m.name}</option>)}</select></label>
            <label className="field full"><span>Caixa de destino opcional</span><select value={movementForm.toBoxId} onChange={(e) => setMovementForm({ ...movementForm, toBoxId: e.target.value })}><option value="">Sem caixa definida</option>{targetBoxes.map((b) => <option value={b.id} key={b.id}>{b.code}</option>)}</select></label>
          </> : <label className="field full"><span>Destino</span><input required value={movementForm.destination} onChange={(e) => setMovementForm({ ...movementForm, destination: e.target.value })} placeholder={movementForm.movementType === "transport" ? "Ex.: feira, exposição, visita técnica" : "Ex.: novo proprietário / local de destino"} /></label>}
          <label className="field full"><span>Observações</span><textarea rows={3} value={movementForm.notes} onChange={(e) => setMovementForm({ ...movementForm, notes: e.target.value })} /></label>
          <div className="form-actions full"><button disabled={busy || !movementForm.colonyId || !movable} type="submit">{busy ? "Salvando..." : "Registrar movimentação"}</button></div>
        </form>
      </section>
      <section className="panel list-panel"><div className="panel-heading"><h2>Histórico da colônia</h2><p>Selecione uma movimentação para anexar ou consultar documentos estruturados.</p></div>
        {!movementForm.colonyId ? <div className="empty-list">Selecione uma colônia.</div> : loading ? <div className="empty-list">Carregando...</div> : movements.length === 0 ? <div className="empty-list">Nenhuma movimentação registrada.</div> : <div className="record-list">{movements.map((item) => <article className="record-card" key={item.id}><div className="record-title-row"><div><strong>{movementLabel(item.movementType)}</strong><span>{formatDateTime(item.movedAt)} · {item.fromMeliponaryName}{item.toMeliponaryName ? ` → ${item.toMeliponaryName}` : item.destination ? ` → ${item.destination}` : ""}</span></div><button type="button" className="button-secondary" onClick={() => setDocumentForm({ ...documentInitial, movementId: item.id })}>Documentos</button></div>{item.notes && <p>{item.notes}</p>}</article>)}</div>}
      </section>
    </div>

    <section className="panel"><div className="panel-heading"><h2>Documentos da movimentação</h2><p>GTA, autorizações, notas, recibos, declarações, protocolos ou outros comprovantes. O arquivo é apenas referenciado por caminho.</p></div>
      {!documentForm.movementId ? <div className="empty-list">Escolha “Documentos” em uma movimentação acima.</div> : <div className="content-grid">
        <form className="form-grid" onSubmit={submitDocument}>
          <label className="field"><span>Tipo</span><select value={documentForm.documentType} onChange={(e) => setDocumentForm({ ...documentForm, documentType: e.target.value })}><option value="gta">GTA</option><option value="authorization">Autorização</option><option value="invoice">Nota fiscal</option><option value="receipt">Recibo</option><option value="declaration">Declaração</option><option value="protocol">Protocolo</option><option value="certificate">Certificado</option><option value="other">Outro</option></select></label>
          <label className="field"><span>Número / referência</span><input required value={documentForm.referenceNumber} onChange={(e) => setDocumentForm({ ...documentForm, referenceNumber: e.target.value })} /></label>
          <label className="field"><span>Sistema de origem</span><input value={documentForm.sourceSystem} onChange={(e) => setDocumentForm({ ...documentForm, sourceSystem: e.target.value })} placeholder="Ex.: GEDAVE" /></label>
          <label className="field"><span>Emissor</span><input value={documentForm.issuer} onChange={(e) => setDocumentForm({ ...documentForm, issuer: e.target.value })} /></label>
          <label className="field"><span>Emissão</span><input type="datetime-local" value={documentForm.issuedAt} onChange={(e) => setDocumentForm({ ...documentForm, issuedAt: e.target.value })} /></label>
          <label className="field"><span>Validade</span><input type="datetime-local" value={documentForm.validUntil} onChange={(e) => setDocumentForm({ ...documentForm, validUntil: e.target.value })} /></label>
          <label className="field full"><span>Caminho do arquivo opcional</span><input value={documentForm.filePath} onChange={(e) => setDocumentForm({ ...documentForm, filePath: e.target.value })} /></label>
          <label className="field full"><span>Observações</span><textarea rows={2} value={documentForm.notes} onChange={(e) => setDocumentForm({ ...documentForm, notes: e.target.value })} /></label>
          <div className="form-actions full"><button disabled={busy} type="submit">{busy ? "Salvando..." : "Vincular documento"}</button></div>
        </form>
        <div>{documents.length === 0 ? <div className="empty-list">Nenhum documento vinculado.</div> : <div className="record-list">{documents.map((doc) => <article className="record-card" key={doc.id}><div className="record-title-row"><div><strong>{documentLabel(doc.documentType)} · {doc.referenceNumber}</strong><span>{doc.sourceSystem || doc.issuer || "Sem emissor informado"}</span></div><span className="badge">{doc.validUntil ? `Até ${formatDateTime(doc.validUntil)}` : "Sem validade"}</span></div>{doc.notes && <p>{doc.notes}</p>}</article>)}</div>}</div>
      </div>}
    </section>
  </div>;
}
function normalizeDateTime(value?: string) { if (!value) return undefined; const n = value.replace("T", " "); return n.length === 16 ? `${n}:00` : n; }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function movementLabel(v: string) { return v === "internal_transfer" ? "Transferência interna" : v === "external_transfer" ? "Transferência externa" : "Transporte"; }
function documentLabel(v: string) { const l: Record<string,string> = { gta:"GTA", authorization:"Autorização", invoice:"Nota fiscal", receipt:"Recibo", declaration:"Declaração", protocol:"Protocolo", certificate:"Certificado", other:"Outro" }; return l[v] || v; }
