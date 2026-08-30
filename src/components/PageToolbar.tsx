import type { ReactNode } from "react";
import { Icon } from "./Icon";

type PageToolbarProps = {
  title: string;
  description?: string;
  count?: string;
  search?: { value: string; onChange: (value: string) => void; placeholder?: string };
  primaryAction?: { label: string; onClick: () => void; disabled?: boolean };
  children?: ReactNode;
};

export function PageToolbar({ title, description, count, search, primaryAction, children }: PageToolbarProps) {
  return <section className="page-toolbar"><div className="page-toolbar-title"><h1>{title}</h1>{description && <p>{description}</p>}</div><div className="page-toolbar-controls">{search && <label className="toolbar-search"><span className="sr-only">Buscar</span><input value={search.value} onChange={(event) => search.onChange(event.target.value)} placeholder={search.placeholder ?? "Buscar..."} /></label>}{children}{count && <span className="toolbar-count">{count}</span>}{primaryAction && <button type="button" className="button-primary" onClick={primaryAction.onClick} disabled={primaryAction.disabled}><Icon name="plus" />{primaryAction.label}</button>}</div></section>;
}
