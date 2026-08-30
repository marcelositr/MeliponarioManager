import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { analyzeSpeciesCsv, type SpeciesImportPreview, type SpeciesImportResult } from "../lib/species-import";
import { publicError } from "../lib/presentation";
import { Dialog } from "./Dialog";

type Props = {
  busy: boolean;
  onImport: (sourcePath: string) => Promise<SpeciesImportResult | null>;
};

export function SpeciesImportControl({ busy, onImport }: Props) {
  const [openDialog, setOpenDialog] = useState(false);
  const [sourcePath, setSourcePath] = useState("");
  const [preview, setPreview] = useState<SpeciesImportPreview | null>(null);
  const [analyzing, setAnalyzing] = useState(false);
  const [error, setError] = useState("");

  function reset() {
    setSourcePath("");
    setPreview(null);
    setAnalyzing(false);
    setError("");
  }

  function close() {
    if (busy || analyzing) return;
    reset();
    setOpenDialog(false);
  }

  async function chooseCsv() {
    const selected = await open({
      multiple: false,
      directory: false,
      title: "Selecionar lista de espécies",
      filters: [{ name: "Lista de espécies (CSV)", extensions: ["csv"] }],
    });
    if (typeof selected !== "string") return;

    setSourcePath(selected);
    setPreview(null);
    setError("");
    setAnalyzing(true);
    try {
      setPreview(await analyzeSpeciesCsv(selected));
    } catch (cause) {
      setError(publicError(cause, "Não foi possível analisar o arquivo CSV."));
    } finally {
      setAnalyzing(false);
    }
  }

  async function confirmImport() {
    if (!sourcePath || !preview || preview.invalidRows > 0 || preview.newRows === 0) return;
    const result = await onImport(sourcePath);
    if (!result) return;
    reset();
    setOpenDialog(false);
  }

  const canImport = Boolean(preview && preview.newRows > 0 && preview.invalidRows === 0 && !analyzing && !busy);

  return <>
    <button className="button-secondary" type="button" onClick={() => setOpenDialog(true)} disabled={busy}>Importar lista…</button>
    <Dialog
      open={openDialog}
      onClose={close}
      title="Importar lista de espécies"
      description="Adiciona espécies ao catálogo local sem alterar cadastros existentes. Duplicadas são ignoradas."
      size="large"
    >
      <div className="page-stack compact-stack">
        <div className="inline-notice">
          Formato esperado: <strong>nome_popular;nome_cientifico;genero;observacoes</strong>. Também são aceitos cabeçalhos equivalentes em inglês e separador por vírgula. Somente o nome popular é obrigatório.
        </div>
        <div className="inline-notice">
          A lista é fornecida pelo usuário. O MeliponarioManager não valida legislação, permissões por estado ou se o conteúdo corresponde a uma lista oficial vigente.
        </div>

        <div className="workspace-actions">
          <button className="button-secondary" type="button" onClick={() => void chooseCsv()} disabled={busy || analyzing}>
            {analyzing ? "Analisando…" : preview ? "Escolher outro CSV…" : "Selecionar CSV…"}
          </button>
          {preview && <span className="toolbar-count">{preview.fileName}</span>}
        </div>

        {error && <div className="inline-notice" role="alert">{error}</div>}

        {preview && <>
          <div className="summary-grid">
            <div className="summary-item"><span>Linhas</span><strong>{preview.totalRows}</strong></div>
            <div className="summary-item"><span>Novas</span><strong>{preview.newRows}</strong></div>
            <div className="summary-item"><span>Duplicadas</span><strong>{preview.duplicateRows}</strong></div>
            <div className="summary-item"><span>Inválidas</span><strong>{preview.invalidRows}</strong></div>
          </div>

          <div className="table-wrap">
            <table className="data-table">
              <thead><tr><th>Linha</th><th>Nome popular</th><th>Nome científico</th><th>Gênero</th><th>Situação</th></tr></thead>
              <tbody>{preview.rows.map((row) => <tr key={row.rowNumber}>
                <td>{row.rowNumber}</td>
                <td><strong>{row.commonName || "Não informado"}</strong></td>
                <td className="scientific-name">{row.scientificName || "—"}</td>
                <td>{row.genus || "—"}</td>
                <td>{row.status === "new" ? "Nova" : row.status === "duplicate" ? "Duplicada · será ignorada" : row.message || "Inválida"}</td>
              </tr>)}</tbody>
            </table>
          </div>

          {preview.truncated && <div className="inline-notice">A prévia mostra as primeiras 50 linhas. A análise e a importação consideram o arquivo completo.</div>}
          {preview.invalidRows > 0 && <div className="inline-notice" role="alert">Corrija as linhas inválidas antes de importar. Nenhuma espécie será gravada enquanto houver erros no arquivo.</div>}
          {preview.newRows === 0 && preview.invalidRows === 0 && <div className="inline-notice">Nenhuma espécie nova foi encontrada. As entradas do arquivo já existem no catálogo.</div>}
        </>}

        <div className="form-actions">
          <button className="button-secondary" type="button" onClick={close} disabled={busy || analyzing}>Cancelar</button>
          <button type="button" onClick={() => void confirmImport()} disabled={!canImport}>{busy ? "Importando…" : preview ? `Importar ${preview.newRows}` : "Importar"}</button>
        </div>
      </div>
    </Dialog>
  </>;
}
