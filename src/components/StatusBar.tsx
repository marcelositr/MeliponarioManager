import { Icon } from "./Icon";

type StatusBarProps = {
  connectionStatus: string;
  activeMeliponaryLabel: string;
  appVersion: string;
};

export function StatusBar({ connectionStatus, activeMeliponaryLabel, appVersion }: StatusBarProps) {
  return <footer className="status-bar"><div className="status-left"><span className="status-item"><Icon name="database" /> Banco local · {connectionStatus}</span><span className="status-divider" aria-hidden="true" /><span className="status-item">Meliponário: {activeMeliponaryLabel}</span></div><span className="status-version">v{appVersion}</span></footer>;
}
