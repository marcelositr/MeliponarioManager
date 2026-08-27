import { getVersion } from "@tauri-apps/api/app";
import { useEffect, useMemo, useState } from "react";
import { Dialog } from "./components/Dialog";
import { Icon } from "./components/Icon";
import { Sidebar } from "./components/Sidebar";
import { StatusBar } from "./components/StatusBar";
import { TopMenu, type ThemeMode } from "./components/TopMenu";
import { changeColonyLifecycle, createBox, createBoxMaintenance, createColony, createColonyDivision, createColonyEvent, createColonyMovement, createFeeding, createInspection, createMeliponary, createMovementDocument, createProductionRecord, createSpecies, deleteInspectionPhoto, importInspectionPhoto, loadCoreData, loadDashboardStats, placeColony } from "./lib/api";
import { AlertsPage } from "./pages/AlertsPage";
import { AssetsPage } from "./pages/AssetsPage";
import { BoxesPage } from "./pages/BoxesPage";
import { ColoniesPage } from "./pages/ColoniesPage";
import { DashboardPage } from "./pages/DashboardPage";
import { DataManagementPage } from "./pages/DataManagementPage";
import { DivisionsPage } from "./pages/DivisionsPage";
import { EventsTimelinePage } from "./pages/EventsTimelinePage";
import { FeedingPage } from "./pages/FeedingPage";
import { InspectionsPage } from "./pages/InspectionsPage";
import { LifecyclePage } from "./pages/LifecyclePage";
import { MeliponariesPage } from "./pages/MeliponariesPage";
import { MovementsPage } from "./pages/MovementsPage";
import { ProductionPage } from "./pages/ProductionPage";
import { SpeciesPage } from "./pages/SpeciesPage";
import type { ChangeColonyLifecycleInput, CoreData, CreateBoxInput, CreateBoxMaintenanceInput, CreateColonyEventInput, CreateColonyInput, CreateDivisionInput, CreateFeedingInput, CreateInspectionInput, CreateMeliponaryInput, CreateMovementDocumentInput, CreateMovementInput, CreateProductionInput, CreateSpeciesInput, DashboardStats, ImportInspectionPhotoInput, PlaceColonyInput, View } from "./types";

const emptyData: CoreData = { meliponaries: [], species: [], colonies: [], boxes: [] };
const emptyStats: DashboardStats = { meliponaries: 0, species: 0, colonies: 0, boxes: 0, inspections: 0, photos: 0, events: 0, divisions: 0, feedings: 0, production: 0, movements: 0, documents: 0, maintenance: 0, lifecycle: 0, alerts: 0 };
const viewTitles: Record<View, string> = {
  dashboard: "Visão geral", meliponaries: "Meliponários", species: "Espécies", colonies: "Colônias", boxes: "Caixas", inspections: "Inspeções", feeding: "Alimentação", production: "Produção", history: "Histórico", alerts: "Alertas", genealogy: "Divisões e genealogia", movements: "Movimentações", assets: "Manutenção e fotos", lifecycle: "Ciclo de vida", data: "Dados e relatórios",
};
type Feedback = { kind: "success" | "error"; text: string } | null;

const THEME_KEY = "meliponario.ui.theme";
const SIDEBAR_KEY = "meliponario.ui.sidebarCollapsed";
const MELIPONARY_KEY = "meliponario.ui.activeMeliponary";

function storedTheme(): ThemeMode {
  const value = localStorage.getItem(THEME_KEY);
  return value === "light" || value === "dark" || value === "system" ? value : "system";
}

function App() {
  const [activeView, setActiveView] = useState<View>("dashboard");
  const [data, setData] = useState<CoreData>(emptyData);
  const [stats, setStats] = useState<DashboardStats>(emptyStats);
  const [connectionStatus, setConnectionStatus] = useState("Conectando...");
  const [appVersion, setAppVersion] = useState("...");
  const [busy, setBusy] = useState(false);
  const [feedback, setFeedback] = useState<Feedback>(null);
  const [theme, setTheme] = useState<ThemeMode>(storedTheme);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(() => localStorage.getItem(SIDEBAR_KEY) === "1");
  const [compactViewport, setCompactViewport] = useState(false);
  const [activeMeliponaryId, setActiveMeliponaryId] = useState(() => localStorage.getItem(MELIPONARY_KEY) || "all");
  const [aboutOpen, setAboutOpen] = useState(false);

  async function refresh() {
    const [coreData, dashboardStats] = await Promise.all([loadCoreData(), loadDashboardStats()]);
    setData(coreData);
    setStats(dashboardStats);
    setConnectionStatus("Conectado");
  }

  useEffect(() => {
    refresh().catch(() => { setConnectionStatus("Abra pelo Tauri"); setFeedback({ kind: "error", text: "Não foi possível acessar o banco local. Execute a aplicação pelo Tauri." }); });
    getVersion().then(setAppVersion).catch(() => setAppVersion("dev"));
  }, []);

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => { document.documentElement.dataset.theme = theme === "system" ? (media.matches ? "dark" : "light") : theme; };
    apply();
    media.addEventListener("change", apply);
    localStorage.setItem(THEME_KEY, theme);
    return () => media.removeEventListener("change", apply);
  }, [theme]);

  useEffect(() => {
    const update = () => setCompactViewport(window.innerWidth < 1080);
    update();
    window.addEventListener("resize", update);
    return () => window.removeEventListener("resize", update);
  }, []);

  useEffect(() => {
    if (activeMeliponaryId !== "all" && !data.meliponaries.some((item) => item.id === activeMeliponaryId)) setActiveMeliponaryId("all");
  }, [activeMeliponaryId, data.meliponaries]);

  const activeMeliponary = useMemo(() => data.meliponaries.find((item) => item.id === activeMeliponaryId) ?? null, [activeMeliponaryId, data.meliponaries]);
  const activeMeliponaryLabel = activeMeliponary?.name ?? "Todos os meliponários";
  const effectiveCollapsed = sidebarCollapsed || compactViewport;

  function changeSidebar() {
    const next = !sidebarCollapsed;
    setSidebarCollapsed(next);
    localStorage.setItem(SIDEBAR_KEY, next ? "1" : "0");
  }

  function changeMeliponary(value: string) {
    setActiveMeliponaryId(value);
    localStorage.setItem(MELIPONARY_KEY, value);
  }

  async function runMutation(action: () => Promise<unknown>, successMessage: string): Promise<boolean> {
    setBusy(true); setFeedback(null);
    try { await action(); await refresh(); setFeedback({ kind: "success", text: successMessage }); return true; }
    catch (error) { setFeedback({ kind: "error", text: readableError(error) }); return false; }
    finally { setBusy(false); }
  }

  const createMeliponaryFromUi = (input: CreateMeliponaryInput) => runMutation(() => createMeliponary(input), "Meliponário cadastrado com sucesso.");
  const createSpeciesFromUi = (input: CreateSpeciesInput) => runMutation(() => createSpecies(input), "Espécie cadastrada com sucesso.");
  const createBoxFromUi = (input: CreateBoxInput) => runMutation(() => createBox(input), "Caixa cadastrada com sucesso.");
  const createColonyFromUi = (input: CreateColonyInput) => runMutation(() => createColony(input), "Colônia cadastrada com sucesso.");
  const placeColonyFromUi = (input: PlaceColonyInput) => runMutation(() => placeColony(input), "Ocupação de caixa registrada e histórico preservado.");
  const createInspectionFromUi = (input: CreateInspectionInput) => runMutation(() => createInspection(input), "Inspeção registrada com sucesso.");
  const createFeedingFromUi = (input: CreateFeedingInput) => runMutation(() => createFeeding(input), "Alimentação registrada com sucesso.");
  const createProductionFromUi = (input: CreateProductionInput) => runMutation(() => createProductionRecord(input), "Produção registrada com sucesso.");
  const createEventFromUi = (input: CreateColonyEventInput) => runMutation(() => createColonyEvent(input), "Evento registrado e incluído na timeline.");
  const createDivisionFromUi = (input: CreateDivisionInput) => runMutation(() => createColonyDivision(input), "Divisão registrada e genealogia atualizada.");
  const createMovementFromUi = (input: CreateMovementInput) => runMutation(() => createColonyMovement(input), "Movimentação registrada e rastreabilidade atualizada.");
  const createMovementDocumentFromUi = (input: CreateMovementDocumentInput) => runMutation(() => createMovementDocument(input), "Documento vinculado à movimentação.");
  const importInspectionPhotoFromUi = (input: ImportInspectionPhotoInput) => runMutation(() => importInspectionPhoto(input), "Foto importada para o armazenamento gerenciado.");
  const deleteInspectionPhotoFromUi = (photoId: string) => runMutation(() => deleteInspectionPhoto(photoId), "Foto removida com segurança.");
  const createBoxMaintenanceFromUi = (input: CreateBoxMaintenanceInput) => runMutation(() => createBoxMaintenance(input), "Manutenção da caixa registrada.");
  const changeLifecycleFromUi = (input: ChangeColonyLifecycleInput) => runMutation(() => changeColonyLifecycle(input), "Ciclo de vida atualizado e histórico preservado.");

  return <div className="application-shell">
    <TopMenu theme={theme} onThemeChange={setTheme} sidebarCollapsed={sidebarCollapsed} onToggleSidebar={changeSidebar} onNavigate={setActiveView} onRefresh={() => { void refresh(); }} onOpenAbout={() => setAboutOpen(true)} />
    <div className={effectiveCollapsed ? "shell-body sidebar-is-collapsed" : "shell-body"}>
      <Sidebar activeView={activeView} onNavigate={setActiveView} collapsed={effectiveCollapsed} onToggle={changeSidebar} />
      <main className="workspace">
        <header className="context-bar">
          <div className="context-title"><span className="topbar-context">Operação</span><strong>{viewTitles[activeView]}</strong></div>
          <div className="context-actions">
            <label className="meliponary-selector"><span>Meliponário</span><select value={activeMeliponaryId} onChange={(event) => changeMeliponary(event.target.value)}><option value="all">Todos os meliponários</option>{data.meliponaries.map((item) => <option key={item.id} value={item.id}>{item.name}</option>)}</select></label>
            <button className="icon-button" type="button" onClick={() => { void refresh(); }} aria-label="Atualizar dados" title="Atualizar"><Icon name="refresh" /></button>
          </div>
        </header>
        {feedback && <div className={`feedback-banner ${feedback.kind}`} role={feedback.kind === "error" ? "alert" : "status"}><span>{feedback.text}</span><button className="icon-button" type="button" onClick={() => setFeedback(null)} aria-label="Fechar aviso"><Icon name="close" /></button></div>}
        <div className="workspace-content">
          {activeView === "dashboard" && <DashboardPage stats={stats} onNavigate={setActiveView} />}
          {activeView === "meliponaries" && <MeliponariesPage items={data.meliponaries} busy={busy} onCreate={createMeliponaryFromUi} />}
          {activeView === "species" && <SpeciesPage items={data.species} busy={busy} onCreate={createSpeciesFromUi} />}
          {activeView === "colonies" && <ColoniesPage items={data.colonies} meliponaries={data.meliponaries} species={data.species} boxes={data.boxes} busy={busy} onCreate={createColonyFromUi} onPlace={placeColonyFromUi} />}
          {activeView === "boxes" && <BoxesPage items={data.boxes} meliponaries={data.meliponaries} busy={busy} onCreate={createBoxFromUi} />}
          {activeView === "inspections" && <InspectionsPage colonies={data.colonies} busy={busy} onCreate={createInspectionFromUi} />}
          {activeView === "feeding" && <FeedingPage colonies={data.colonies} busy={busy} onCreate={createFeedingFromUi} />}
          {activeView === "production" && <ProductionPage colonies={data.colonies} busy={busy} onCreate={createProductionFromUi} />}
          {activeView === "history" && <EventsTimelinePage colonies={data.colonies} busy={busy} onCreate={createEventFromUi} />}
          {activeView === "alerts" && <AlertsPage />}
          {activeView === "genealogy" && <DivisionsPage colonies={data.colonies} busy={busy} onCreate={createDivisionFromUi} />}
          {activeView === "movements" && <MovementsPage colonies={data.colonies} meliponaries={data.meliponaries} boxes={data.boxes} busy={busy} onCreateMovement={createMovementFromUi} onCreateDocument={createMovementDocumentFromUi} />}
          {activeView === "assets" && <AssetsPage colonies={data.colonies} boxes={data.boxes} busy={busy} onImportPhoto={importInspectionPhotoFromUi} onDeletePhoto={deleteInspectionPhotoFromUi} onCreateMaintenance={createBoxMaintenanceFromUi} />}
          {activeView === "lifecycle" && <LifecyclePage colonies={data.colonies} busy={busy} onChange={changeLifecycleFromUi} />}
          {activeView === "data" && <DataManagementPage />}
        </div>
      </main>
    </div>
    <StatusBar connectionStatus={connectionStatus} activeMeliponaryLabel={activeMeliponaryLabel} appVersion={appVersion} />
    <Dialog open={aboutOpen} onClose={() => setAboutOpen(false)} title="Sobre o MeliponarioManager" description="Gestão local e rastreável do meliponário." size="small"><div className="about-dialog"><p>Aplicação desktop local-first para controle de plantel, manejo e histórico operacional.</p><dl><div><dt>Versão</dt><dd>v{appVersion}</dd></div><div><dt>Banco</dt><dd>SQLite local</dd></div></dl><div className="dialog-actions"><button type="button" onClick={() => setAboutOpen(false)}>Fechar</button></div></div></Dialog>
  </div>;
}

function readableError(error: unknown) { if (typeof error === "string" && error.trim()) return error; if (error instanceof Error && error.message) return error.message; return "Não foi possível concluir a operação."; }
export default App;
