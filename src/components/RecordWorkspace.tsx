import type { ReactNode } from "react";

type RecordWorkspaceProps = {
  backLabel: string;
  title: string;
  subtitle?: string;
  children: ReactNode;
  onBack: () => void;
};

export function RecordWorkspace({ backLabel, title, subtitle, children, onBack }: RecordWorkspaceProps) {
  return <div className="record-workspace">
    <button className="record-back" type="button" onClick={onBack}>← {backLabel}</button>
    <header className="record-workspace-header"><div><h1>{title}</h1>{subtitle && <p>{subtitle}</p>}</div></header>
    <nav className="record-tabs" aria-label="Seções do registro"><button className="active" type="button" aria-current="page">Resumo</button></nav>
    <section className="record-workspace-body">{children}</section>
  </div>;
}
