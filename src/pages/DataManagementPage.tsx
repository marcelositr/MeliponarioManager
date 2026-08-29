import { confirm, open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { Icon } from "../components/Icon";
import { createFullBackup, exportPortableJson, generateManagementReport, stageRestore } from "../lib/data-api";
import { publicError } from "../lib/presentation";

type ResultState = { kind: "success" | "error"; text: string } | null;

export function DataManagementPage() {
  const [busy, setBusy] = useState("");
  const [restorePath, setRestorePath] = useState("");
  const [result, setResult] = useState<ResultState>(null);

  async function generate(kind: "backup" | "json" | "report") {
    setBusy(kind); setResult(null);
    try {
      const artifact = kind === "backup" ? await createFullBackup() : kind === "json" ? await exportPortableJson() : await generateManagementReport();
      setResult({ kind: "success", text: `${artifactLabel(kind)} criado em: ${artifact.path}` });
    } catch (error) {
      setResult({ kind: "error", text: publicError(error) });
    } finally { setBusy(""); }
  }

  async function selectRestoreDirectory() {
    const selected = await open({ directory: true, multiple: false, title: "Selecionar backup completo" });
    if (typeof selected === "string") setRestorePath(selected);
  }

  async function selectRestoreDatabase() {
    const selected = await open({
      multiple: false,
      directory: false,
      title: "Selecionar banco para restauração",
      filters: [{ name: "Banco SQLite", extensions: ["db"] }],
    });
    if (typeof selected === "string") setRestorePath(selected);
  }

  async function prepareRestore() {
    if (!restorePath.trim()) return;
    const approved = await confirm(
      "Preparar esta restauração? Ela só será aplicada na próxima abertura do aplicativo e o estado atual será salvo antes da troca.",
      { title: "Preparar restauração", kind: "warning" },
    );
    if (!approved) return;
    setBusy("restore"); setResult(null);
    try {
      const staged = await stageRestore(restorePath);
      setResult({ kind: "success", text: `${staged.message}${staged.includesMedia ? " O backup inclui a pasta de mídia." : " O backup não inclui mídia; a mídia atual será preservada."}` });
    } catch (error) {
      setResult({ kind: "error", text: publicError(error, "Não foi possível validar e preparar a restauração.") });
    } finally { setBusy(""); }
  }

  return (
    <div className="page-stack">
      <section className="page-heading"><div><span className="eyebrow">Dados locais</span><h1>Backup, exportação e relatórios</h1><p>Proteja o banco e a mídia, gere uma exportação legível e prepare restaurações sem substituir um banco que esteja aberto.</p></div></section>

      {result && <div className={`feedback-banner ${result.kind}`} role={result.kind === "error" ? "alert" : "status"}><span>{result.text}</span><button className="icon-button" type="button" onClick={() => setResult(null)} aria-label="Fechar aviso"><Icon name="close" /></button></div>}

      <div className="content-grid">
        <section className="panel">
          <div className="panel-heading"><h2>Backup completo</h2><p>Cria uma cópia consistente do SQLite e inclui a pasta de fotos gerenciada.</p></div>
          <div className="form-actions"><button type="button" onClick={() => void generate("backup")} disabled={Boolean(busy)}>{busy === "backup" ? "Criando backup..." : "Criar backup completo"}</button></div>
        </section>
        <section className="panel">
          <div className="panel-heading"><h2>Exportação portátil</h2><p>Gera JSON com cadastros, ocupações e timeline de cada colônia para consulta e interoperabilidade futura.</p></div>
          <div className="form-actions"><button type="button" onClick={() => void generate("json")} disabled={Boolean(busy)}>{busy === "json" ? "Exportando..." : "Exportar JSON"}</button></div>
        </section>
      </div>

      <div className="content-grid">
        <section className="panel">
          <div className="panel-heading"><h2>Relatório gerencial</h2><p>Gera Markdown com estrutura, situação do plantel, espécies, ocupação e pendências atuais.</p></div>
          <div className="form-actions"><button type="button" onClick={() => void generate("report")} disabled={Boolean(busy)}>{busy === "report" ? "Gerando..." : "Gerar relatório"}</button></div>
        </section>
        <section className="panel form-panel">
          <div className="panel-heading"><h2>Preparar restauração</h2><p>Selecione uma pasta de backup completo ou um arquivo meliponario.db. A integridade é validada antes de qualquer troca.</p></div>
          <div className="form-grid">
            <label className="field full"><span>Backup selecionado</span><input value={restorePath} readOnly placeholder="Nenhum backup selecionado" /></label>
            <div className="workspace-actions field full" aria-label="Selecionar origem da restauração">
              <button type="button" className="button-secondary" onClick={() => void selectRestoreDirectory()} disabled={Boolean(busy)}>Selecionar pasta…</button>
              <button type="button" className="button-secondary" onClick={() => void selectRestoreDatabase()} disabled={Boolean(busy)}>Selecionar meliponario.db…</button>
            </div>
            <div className="inline-notice field full" role="status">A restauração preparada só entra em vigor quando o aplicativo for fechado e aberto novamente. Antes da troca, o estado atual recebe um backup de segurança automático.</div>
            <div className="form-actions full"><button type="button" onClick={() => void prepareRestore()} disabled={Boolean(busy) || !restorePath.trim()}>{busy === "restore" ? "Validando..." : "Validar e preparar restauração"}</button></div>
          </div>
        </section>
      </div>
    </div>
  );
}

function artifactLabel(kind: "backup" | "json" | "report") { return kind === "backup" ? "Backup" : kind === "json" ? "Exportação" : "Relatório"; }
