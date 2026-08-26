import { useEffect, useState } from "react";
import { Sidebar } from "./components/Sidebar";
import {
  createBox,
  createColony,
  createMeliponary,
  createSpecies,
  loadCoreData,
  loadDashboardStats,
  placeColony,
} from "./lib/api";
import { BoxesPage } from "./pages/BoxesPage";
import { ColoniesPage } from "./pages/ColoniesPage";
import { DashboardPage } from "./pages/DashboardPage";
import { MeliponariesPage } from "./pages/MeliponariesPage";
import { SpeciesPage } from "./pages/SpeciesPage";
import type {
  CoreData,
  CreateBoxInput,
  CreateColonyInput,
  CreateMeliponaryInput,
  CreateSpeciesInput,
  DashboardStats,
  PlaceColonyInput,
  View,
} from "./types";

const emptyData: CoreData = { meliponaries: [], species: [], colonies: [], boxes: [] };
const emptyStats: DashboardStats = {
  meliponaries: 0,
  species: 0,
  colonies: 0,
  boxes: 0,
  inspections: 0,
  photos: 0,
  events: 0,
  divisions: 0,
  feedings: 0,
  production: 0,
  movements: 0,
  documents: 0,
  maintenance: 0,
  lifecycle: 0,
  alerts: 0,
};

const viewTitles: Record<View, string> = {
  dashboard: "Visão geral",
  meliponaries: "Meliponários",
  species: "Espécies",
  colonies: "Colônias",
  boxes: "Caixas",
};

type Feedback = { kind: "success" | "error"; text: string } | null;

function App() {
  const [activeView, setActiveView] = useState<View>("dashboard");
  const [data, setData] = useState<CoreData>(emptyData);
  const [stats, setStats] = useState<DashboardStats>(emptyStats);
  const [connectionStatus, setConnectionStatus] = useState("Conectando...");
  const [busy, setBusy] = useState(false);
  const [feedback, setFeedback] = useState<Feedback>(null);

  async function refresh() {
    const [coreData, dashboardStats] = await Promise.all([loadCoreData(), loadDashboardStats()]);
    setData(coreData);
    setStats(dashboardStats);
    setConnectionStatus("Conectado");
  }

  useEffect(() => {
    refresh().catch(() => {
      setConnectionStatus("Abra pelo Tauri");
      setFeedback({ kind: "error", text: "Não foi possível acessar o banco local. Execute a aplicação pelo Tauri." });
    });
  }, []);

  async function runMutation(action: () => Promise<unknown>, successMessage: string): Promise<boolean> {
    setBusy(true);
    setFeedback(null);
    try {
      await action();
      await refresh();
      setFeedback({ kind: "success", text: successMessage });
      return true;
    } catch (error) {
      setFeedback({ kind: "error", text: readableError(error) });
      return false;
    } finally {
      setBusy(false);
    }
  }

  const createMeliponaryFromUi = (input: CreateMeliponaryInput) =>
    runMutation(() => createMeliponary(input), "Meliponário cadastrado com sucesso.");

  const createSpeciesFromUi = (input: CreateSpeciesInput) =>
    runMutation(() => createSpecies(input), "Espécie cadastrada com sucesso.");

  const createBoxFromUi = (input: CreateBoxInput) =>
    runMutation(() => createBox(input), "Caixa cadastrada com sucesso.");

  const createColonyFromUi = (input: CreateColonyInput) =>
    runMutation(() => createColony(input), "Colônia cadastrada com sucesso.");

  const placeColonyFromUi = (input: PlaceColonyInput) =>
    runMutation(() => placeColony(input), "Ocupação de caixa registrada e histórico preservado.");

  return (
    <div className="application-frame">
      <Sidebar activeView={activeView} onNavigate={setActiveView} connectionStatus={connectionStatus} />

      <main className="workspace">
        <header className="workspace-topbar">
          <div>
            <span className="topbar-context">MeliponarioManager</span>
            <strong>{viewTitles[activeView]}</strong>
          </div>
          <span className="version">v0.1.0</span>
        </header>

        {feedback && (
          <div className={`feedback-banner ${feedback.kind}`} role={feedback.kind === "error" ? "alert" : "status"}>
            <span>{feedback.text}</span>
            <button type="button" onClick={() => setFeedback(null)} aria-label="Fechar aviso">×</button>
          </div>
        )}

        <div className="workspace-content">
          {activeView === "dashboard" && <DashboardPage stats={stats} onNavigate={setActiveView} />}
          {activeView === "meliponaries" && (
            <MeliponariesPage items={data.meliponaries} busy={busy} onCreate={createMeliponaryFromUi} />
          )}
          {activeView === "species" && (
            <SpeciesPage items={data.species} busy={busy} onCreate={createSpeciesFromUi} />
          )}
          {activeView === "colonies" && (
            <ColoniesPage
              items={data.colonies}
              meliponaries={data.meliponaries}
              species={data.species}
              boxes={data.boxes}
              busy={busy}
              onCreate={createColonyFromUi}
              onPlace={placeColonyFromUi}
            />
          )}
          {activeView === "boxes" && (
            <BoxesPage items={data.boxes} meliponaries={data.meliponaries} busy={busy} onCreate={createBoxFromUi} />
          )}
        </div>
      </main>
    </div>
  );
}

function readableError(error: unknown) {
  if (typeof error === "string" && error.trim()) return error;
  if (error instanceof Error && error.message) return error.message;
  return "Não foi possível concluir a operação.";
}

export default App;
