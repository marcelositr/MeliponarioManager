import { open } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  importMeliponaryAttachment,
  listMeliponaryAttachments,
  openManagedAttachment,
  removeMeliponaryAttachment,
  revealManagedAttachment,
  updateMeliponaryAttachment,
  type ManagedAttachment,
} from "../lib/files-api";
import { createLatestRequestController, runLatestRequest } from "../lib/latest-request";
import { formatDateTimeBr, publicError } from "../lib/presentation";
import { ConfirmDialog } from "./ConfirmDialog";
import { Dialog } from "./Dialog";
import { RecordActions } from "./RecordActions";

type Feedback = { kind: "success" | "error"; text: string };

export function MeliponaryFilesPanel({ meliponaryId }: { meliponaryId: string }) {
  const [items, setItems] = useState<ManagedAttachment[]>([]);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [feedback, setFeedback] = useState<Feedback | null>(null);
  const [editing, setEditing] = useState<ManagedAttachment | null>(null);
  const [description, setDescription] = useState("");
  const [notes, setNotes] = useState("");
  const [removing, setRemoving] = useState<ManagedAttachment | null>(null);
  const fileRequests = useRef(createLatestRequestController());

  const reload = useCallback(async () => {
    setLoading(true);
    return runLatestRequest(
      fileRequests.current,
      () => listMeliponaryAttachments(meliponaryId),
      {
        onSuccess: setItems,
        onError: (error) => {
          setFeedback({ kind: "error", text: publicError(error, "Não foi possível carregar os arquivos deste meliponário.") });
        },
        onSettled: () => setLoading(false),
      },
    );
  }, [meliponaryId]);

  useEffect(() => {
    setFeedback(null);
    void reload();
    return () => fileRequests.current.invalidate();
  }, [reload]);

  async function attachFile() {
    if (busy) return;
    const selected = await open({ multiple: false, directory: false, title: "Anexar arquivo ao meliponário" });
    if (typeof selected !== "string") return;
    setBusy(true);
    setFeedback(null);
    try {
      await importMeliponaryAttachment({ meliponaryId, sourcePath: selected });
      const refresh = await reload();
      if (refresh === "success") {
        setFeedback({ kind: "success", text: "Arquivo anexado e copiado para a área gerenciada." });
      } else if (refresh === "error") {
        setFeedback({ kind: "error", text: "O arquivo foi anexado, mas a lista não pôde ser atualizada. Reabra esta ficha antes de repetir a operação." });
      }
    } catch (error) {
      setFeedback({ kind: "error", text: publicError(error, "Não foi possível anexar o arquivo.") });
    } finally {
      setBusy(false);
    }
  }

  async function runFileAction(action: "open" | "reveal", item: ManagedAttachment) {
    setFeedback(null);
    try {
      if (action === "open") await openManagedAttachment(item.id);
      else await revealManagedAttachment(item.id);
    } catch (error) {
      setFeedback({ kind: "error", text: publicError(error, action === "open" ? "Não foi possível abrir o arquivo." : "Não foi possível mostrar o arquivo no local.") });
      await reload();
    }
  }

  function beginEdit(item: ManagedAttachment) {
    setEditing(item);
    setDescription(item.description || "");
    setNotes(item.notes || "");
  }

  async function saveEdit(event: React.FormEvent) {
    event.preventDefault();
    if (!editing || busy) return;
    setBusy(true);
    setFeedback(null);
    try {
      await updateMeliponaryAttachment({ id: editing.id, description, notes });
      setEditing(null);
      const refresh = await reload();
      if (refresh === "success") {
        setFeedback({ kind: "success", text: "Descrição do arquivo atualizada." });
      } else if (refresh === "error") {
        setFeedback({ kind: "error", text: "A descrição foi salva, mas a lista não pôde ser atualizada. Reabra esta ficha antes de editar novamente." });
      }
    } catch (error) {
      setFeedback({ kind: "error", text: publicError(error, "Não foi possível atualizar o arquivo.") });
    } finally {
      setBusy(false);
    }
  }

  async function confirmRemove() {
    if (!removing || busy) return;
    setBusy(true);
    setFeedback(null);
    try {
      await removeMeliponaryAttachment(removing.id);
      setRemoving(null);
      const refresh = await reload();
      if (refresh === "success") {
        setFeedback({ kind: "success", text: "Anexo gerenciado removido. O arquivo original não foi alterado." });
      } else if (refresh === "error") {
        setFeedback({ kind: "error", text: "O anexo foi removido, mas a lista não pôde ser atualizada. Reabra esta ficha antes de repetir a operação." });
      }
    } catch (error) {
      setFeedback({ kind: "error", text: publicError(error, "Não foi possível remover o anexo.") });
    } finally {
      setBusy(false);
    }
  }

  return <section className="panel wide-list managed-files-panel">
    <div className="panel-heading panel-heading-actions">
      <div><h2>Arquivos</h2><p>Documentos e outros arquivos associados a este meliponário. As cópias ficam na área gerenciada da aplicação.</p></div>
      <button type="button" onClick={() => void attachFile()} disabled={busy}>Anexar arquivo…</button>
    </div>

    {feedback && <div className={`feedback-banner ${feedback.kind}`} role={feedback.kind === "error" ? "alert" : "status"}>{feedback.text}</div>}
    {loading ? <div className="empty-list" role="status">Carregando arquivos…</div> : items.length === 0 ? <div className="empty-list">Nenhum arquivo anexado a este meliponário.</div> : <div className="table-wrap"><table className="data-table managed-files-table"><thead><tr><th>Arquivo</th><th>Tipo</th><th>Tamanho</th><th>Incluído em</th><th>Descrição</th><th><span className="sr-only">Ações</span></th></tr></thead><tbody>{items.map((item) => <tr key={item.id} className={!item.fileExists ? "attention-row" : undefined}><td><strong>{item.originalName}</strong>{!item.fileExists && <span className="file-missing-label">Arquivo não encontrado</span>}</td><td>{fileTypeLabel(item)}</td><td>{formatFileSize(item.byteSize)}</td><td>{formatDateTimeBr(item.createdAt)}</td><td>{item.description || "—"}</td><td><RecordActions onOpen={item.fileExists ? () => void runFileAction("open", item) : undefined} secondary={[{ label: "Mostrar no local", onClick: () => void runFileAction("reveal", item), disabled: !item.fileExists }, { label: "Editar descrição…", onClick: () => beginEdit(item) }, { label: "Remover…", onClick: () => setRemoving(item), danger: true }]} busy={busy} /></td></tr>)}</tbody></table></div>}

    <Dialog open={Boolean(editing)} title="Editar descrição do arquivo" description={editing?.originalName} onClose={() => !busy && setEditing(null)} size="small">
      <form onSubmit={(event) => void saveEdit(event)} className="form-grid">
        <label className="field"><span>Descrição</span><input autoFocus value={description} onChange={(event) => setDescription(event.target.value)} maxLength={240} /></label>
        <label className="field"><span>Observações</span><textarea value={notes} onChange={(event) => setNotes(event.target.value)} rows={4} maxLength={1000} /></label>
        <div className="form-actions"><button className="button-secondary" type="button" onClick={() => setEditing(null)} disabled={busy}>Cancelar</button><button type="submit" disabled={busy}>{busy ? "Salvando…" : "Salvar"}</button></div>
      </form>
    </Dialog>

    <ConfirmDialog open={Boolean(removing)} title="Remover anexo?" consequence={`A cópia gerenciada de “${removing?.originalName || "arquivo"}” e seus metadados serão removidos. O arquivo original usado na importação não será alterado.`} confirmLabel="Remover anexo" danger busy={busy} onCancel={() => !busy && setRemoving(null)} onConfirm={() => void confirmRemove()} />
  </section>;
}

function formatFileSize(bytes: number) {
  if (!Number.isFinite(bytes) || bytes < 0) return "—";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(bytes < 10 * 1024 ? 1 : 0)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(bytes < 10 * 1024 * 1024 ? 1 : 0)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function fileTypeLabel(item: ManagedAttachment) {
  const extension = item.extension?.toLowerCase();
  if (extension === "pdf") return "PDF";
  if (["jpg", "jpeg", "png", "webp", "gif"].includes(extension || "")) return "Imagem";
  if (["doc", "docx", "odt", "rtf"].includes(extension || "")) return "Documento";
  if (["xls", "xlsx", "ods", "csv"].includes(extension || "")) return "Planilha";
  if (["txt", "md", "json", "xml"].includes(extension || "")) return "Texto";
  return extension ? extension.toUpperCase() : "Arquivo";
}
