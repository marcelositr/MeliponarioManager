import { getVersion } from "@tauri-apps/api/app";
import { useEffect, useMemo, useState } from "react";
import { Dialog } from "./components/Dialog";
import { Icon } from "./components/Icon";
import { Sidebar } from "./components/Sidebar";
import { StatusBar } from "./components/StatusBar";
import { TopMenu } from "./components/TopMenu";
import { WorkspaceRouter } from "./components/WorkspaceRouter";
import { useAppData } from "./hooks/useAppData";
import { normalizeActiveMeliponary, normalizeTheme, readSidebarCollapsed, resolveTheme, UI_STORAGE, type ThemeMode } from "./lib/ui-preferences";
import type { View } from "./types";

const viewTitles: Record<View, string> = {
  dashboard: "Visão geral", agenda: "Agenda", meliponaries: "Meliponários", species: "Espécies", colonies: "Colônias", boxes: "Caixas", inspections: "Inspeções", feeding: "Alimentação", production: "Produção", history: "Histórico", alerts: "Alertas", genealogy: "Divisões e genealogia", movements: "Movimentações", assets: "Manutenção", lifecycle: "Ciclo de vida", data: "Dados e relatórios",
};

function App() {
  const { data, stats, recordStateMap, connectionStatus, busy, feedback, setFeedback, refresh, actions } = useAppData();
  const [activeView, setActiveView] = useState<View>("dashboard");
  const [appVersion, setAppVersion] = useState("...");
  const [theme, setTheme] = useState<ThemeMode>(() => normalizeTheme(localStorage.getItem(UI_STORAGE.theme)));
  const [sidebarCollapsed, setSidebarCollapsed] = useState(() => readSidebarCollapsed(localStorage.getItem(UI_STORAGE.sidebarCollapsed)));
  const [compactViewport, setCompactViewport] = useState(false);
  const [compactSidebarOverride, setCompactSidebarOverride] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [activeMeliponaryId, setActiveMeliponaryId] = useState(() => localStorage.getItem(UI_STORAGE.activeMeliponary) || "all");
  const [aboutOpen, setAboutOpen] = useState(false);

  useEffect(() => { getVersion().then(setAppVersion).catch(() => setAppVersion("dev")); }, []);

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => { document.documentElement.dataset.theme = resolveTheme(theme, media.matches); };
    apply();
    media.addEventListener("change", apply);
    localStorage.setItem(UI_STORAGE.theme, theme);
    return () => media.removeEventListener("change", apply);
  }, [theme]);

  useEffect(() => {
    const update = () => {
      const compact = window.innerWidth < 1080;
      setCompactViewport(compact);
      if (!compact) setCompactSidebarOverride(false);
    };
    update();
    window.addEventListener("resize", update);
    return () => window.removeEventListener("resize", update);
  }, []);

  useEffect(() => {
    const valid = normalizeActiveMeliponary(activeMeliponaryId, data.meliponaries.map((item) => item.id));
    if (valid !== activeMeliponaryId) setActiveMeliponaryId(valid);
  }, [activeMeliponaryId, data.meliponaries]);

  const activeMeliponary = useMemo(() => data.meliponaries.find((item) => item.id === activeMeliponaryId) ?? null, [activeMeliponaryId, data.meliponaries]);
  const activeMeliponaryLabel = activeMeliponary?.name ?? "Todos os meliponários";
  const scopedMeliponaryId = activeMeliponaryId === "all" ? "" : activeMeliponaryId;
  const effectiveCollapsed = sidebarCollapsed || (compactViewport && !compactSidebarOverride);

  function changeSidebar() {
    if (effectiveCollapsed) {
      if (sidebarCollapsed) {
        setSidebarCollapsed(false);
        localStorage.setItem(UI_STORAGE.sidebarCollapsed, "0");
      }
      if (compactViewport) setCompactSidebarOverride(true);
      return;
    }

    setSidebarCollapsed(true);
    localStorage.setItem(UI_STORAGE.sidebarCollapsed, "1");
    setCompactSidebarOverride(false);
  }

  async function handleRefresh() {
    if (refreshing) return;
    setRefreshing(true);
    try {
      await refresh();
      setFeedback({ kind: "success", text: "Dados atualizados." });
    } catch {
      setFeedback({ kind: "error", text: "Não foi possível atualizar os dados. Tente novamente." });
    } finally {
      setRefreshing(false);
    }
  }

  function changeMeliponary(value: string) {
    setActiveMeliponaryId(value);
    localStorage.setItem(UI_STORAGE.activeMeliponary, value);
  }

  return <div className="application-shell">
    <TopMenu theme={theme} onThemeChange={setTheme} sidebarCollapsed={effectiveCollapsed} onToggleSidebar={changeSidebar} onNavigate={setActiveView} onRefresh={() => { void handleRefresh(); }} refreshDisabled={refreshing} onOpenAbout={() => setAboutOpen(true)} />
    <div className={effectiveCollapsed ? "shell-body sidebar-is-collapsed" : "shell-body"}>
      <Sidebar activeView={activeView} onNavigate={setActiveView} collapsed={effectiveCollapsed} onToggle={changeSidebar} />
      <main className="workspace">
        <header className="context-bar">
          <div className="context-title"><span className="topbar-context">Operação</span><strong>{viewTitles[activeView]}</strong></div>
          <div className="context-actions">
            <label className="meliponary-selector"><span>Meliponário</span><select value={activeMeliponaryId} onChange={(event) => changeMeliponary(event.target.value)}><option value="all">Todos os meliponários</option>{data.meliponaries.map((item) => <option key={item.id} value={item.id}>{item.name}</option>)}</select></label>
            <button className="icon-button" type="button" onClick={() => { void handleRefresh(); }} disabled={refreshing} aria-label={refreshing ? "Atualizando dados" : "Atualizar dados"} title={refreshing ? "Atualizando…" : "Atualizar"}><Icon name="refresh" /></button>
          </div>
        </header>
        {feedback && <div className={`feedback-banner ${feedback.kind}`} role={feedback.kind === "error" ? "alert" : "status"}><span>{feedback.text}</span><button className="icon-button" type="button" onClick={() => setFeedback(null)} aria-label="Fechar aviso"><Icon name="close" /></button></div>}
        <div className="workspace-content"><WorkspaceRouter activeView={activeView} activeMeliponaryId={scopedMeliponaryId} data={data} stats={stats} busy={busy} actions={actions} recordStateMap={recordStateMap} onNavigate={setActiveView} /></div>
      </main>
    </div>
    <StatusBar connectionStatus={connectionStatus} activeMeliponaryLabel={activeMeliponaryLabel} appVersion={appVersion} />
    <Dialog open={aboutOpen} onClose={() => setAboutOpen(false)} title="Sobre o MeliponarioManager" description="Gestão local e rastreável do meliponário." size="small"><div className="about-dialog"><p>Aplicação desktop local-first para controle de plantel, manejo e histórico operacional.</p><dl><div><dt>Versão</dt><dd>v{appVersion}</dd></div><div><dt>Banco</dt><dd>SQLite local</dd></div></dl><div className="dialog-actions"><button type="button" onClick={() => setAboutOpen(false)}>Fechar</button></div></div></Dialog>
  </div>;
}

export default App;
