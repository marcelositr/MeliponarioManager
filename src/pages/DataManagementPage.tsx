import { confirm, open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { Icon } from "../components/Icon";
import { createFullBackup, exportPortableJson, stageRestore } from "../lib/data-api";
import { publicError } from "../lib/presentation";

type ResultState = { kind: "success" | "error"; text: string } | null;

export function DataManagementPage() {
  const [busy, setBusy] = useState("");
  const [restorePath, setRestorePath] = useState("");
  const [result, setResult] = useState<ResultState>(null);

  async function generate(kind: "backup" | "json") {
    setBusy(kind); setResult(null);
    try {
      const artifact = kind === "backup" ? await createFullBackup() : await exportPortableJson();
      setResult({ kind: "success", text: `${kind === "backup" ? "Backup" : "Exportação"} criado em: ${artifact.path}` });
    } catch (error) {
      setResult({ kind: "error", text: publicError(error) });
    } finally { setBusy(""); }
  }

  async function selectRestoreDirectory() {
    const selected = await open({ directory: true, multiple: false, title: "Selecionar backup completo" });
    if (typeof selected === "string") setRestorePath(selected);
  }

  async function selectRestoreDatabase() {
    const selected = await open({ multiple: false, directory: false, title: "Selecionar banco para restauração", filters: [{ name: "Banco SQLite", extensions: ["db"] }] });
    if (typeof selected === "string") setRestorePath(selected);
  }

  async function prepareRestore() {
    if (!restorePath.trim()) return;
    const approved = await confirm("Preparar esta restauração? Ela só será aplicada na próxima abertura do aplicativo e o estado atual será salvo antes da troca.", { title: "Preparar restauração", kind: "warning" });
    if (!approved) return;
    setBusy("restore"); setResult(null);
    try {
      const staged = await stageRestore(restorePath);
      setResult({ kind: "success", text: `${staged.message}${staged.includesMedia ? " O backup inclui a pasta de mídia." : " O backup não inclui mídia; a mídia atual será preservada."}` });
    } catch (error) {
      setResult({ kind: "error", text: publicError(error, "Não foi possível validar e preparar a restauração.") });
    } finally { setBusy(""); }
  }

  return <div className="page-stack">
    <section className="page-heading"><div><span className="eyebrow">Dados locais</span><h1>Backup, restauração e exportação</h1><p>Administre cópias de segurança e a exportação estrutural do sistema. Relatórios operacionais ficam na área própria de Relatórios.</p></div></section>
    {result && <div className={`feedback-banner ${result.kind}`} role={result.kind === "error" ? "alert" : "status"}><span>{result.text}</span><button className="icon-button" type="button" onClick={() => setResult(null)} aria-label="Fechar aviso"><Icon name="close" /></button></div>}
    <div className="content-grid">
      <section className="panel"><div className="panel-heading"><h2>Backup completo</h2><p>Cria uma cópia consistente do SQLite e inclui a pasta de fotos gerenciada.</p></div><div className="form-actions"><button type="button" onClick={() => void generate("backup")} disabled={Boolean(busy)}>{busy === "backup" ? "Criando backup..." : "Criar backup completo"}</button></div></section>
      <section className="panel"><div className="panel-heading"><h2>Exportação portátil</h2><p>Gera JSON com a estrutura de dados para interoperabilidade. CSV de relatórios é uma saída separada para planilhas.</p></div><div className="form-actions"><button type="button" onClick={() => void generate("json")} disabled={Boolean(busy)}>{busy === "json" ? "Exportando..." : "Exportar JSON"}</button></div></section>
    </div>
    <section className="panel form-panel"><div className="panel-heading"><h2>Preparar restauração</h2><p>Selecione uma pasta de backup completo ou um arquivo meliponario.db. A integridade é validada antes de qualquer troca.</p></div><div className="form-grid"><label className="field full"><span>Backup selecionado</span><input value={restorePath} readOnly placeholder="Nenhum backup selecionado" /></label><div className="workspace-actions field full" aria-label="Selecionar origem da restauração"><button type="button" className="button-secondary" onClick={() => void selectRestoreDirectory()} disabled={Boolean(busy)}>Selecionar pasta…</button><button type="button" className="button-secondary" onClick={() => void selectRestoreDatabase()} disabled={Boolean(busy)}>Selecionar meliponario.db…</button></div><div className="inline-notice field full" role="status">A restauração preparada só entra em vigor quando o aplicativo for fechado e aberto novamente. Antes da troca, o estado atual recebe um backup de segurança automático.</div><div className="form-actions full"><button type="button" onClick={() => void prepareRestore()} disabled={Boolean(busy) || !restorePath.trim()}>{busy === "restore" ? "Validando..." : "Validar e preparar restauração"}</button></div></div></section>
  </div>;
}
