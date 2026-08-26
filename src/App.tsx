import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type CoreSummary = {
  meliponaries: number;
  species: number;
  colonies: number;
  boxes: number;
};

const emptySummary: CoreSummary = {
  meliponaries: 0,
  species: 0,
  colonies: 0,
  boxes: 0,
};

function App() {
  const [summary, setSummary] = useState<CoreSummary>(emptySummary);
  const [inspectionCount, setInspectionCount] = useState(0);
  const [photoCount, setPhotoCount] = useState(0);
  const [eventCount, setEventCount] = useState(0);
  const [divisionCount, setDivisionCount] = useState(0);
  const [feedingCount, setFeedingCount] = useState(0);
  const [productionCount, setProductionCount] = useState(0);
  const [movementCount, setMovementCount] = useState(0);
  const [movementDocumentCount, setMovementDocumentCount] = useState(0);
  const [alertCount, setAlertCount] = useState(0);
  const [maintenanceCount, setMaintenanceCount] = useState(0);
  const [lifecycleCount, setLifecycleCount] = useState(0);
  const [status, setStatus] = useState("Carregando dados locais...");

  useEffect(() => {
    Promise.all([
      invoke<CoreSummary>("get_core_summary"),
      invoke<number>("get_inspection_count"),
      invoke<number>("get_inspection_photo_count"),
      invoke<number>("get_event_count"),
      invoke<number>("get_division_count"),
      invoke<number>("get_feeding_count"),
      invoke<number>("get_production_count"),
      invoke<number>("get_movement_count"),
      invoke<number>("get_movement_document_count"),
      invoke<number>("get_alert_count"),
      invoke<number>("get_box_maintenance_count"),
      invoke<number>("get_lifecycle_count"),
    ])
      .then(([data, inspections, photos, events, divisions, feedings, production, movements, movementDocuments, alerts, maintenance, lifecycle]) => {
        setSummary(data);
        setInspectionCount(inspections);
        setPhotoCount(photos);
        setEventCount(events);
        setDivisionCount(divisions);
        setFeedingCount(feedings);
        setProductionCount(production);
        setMovementCount(movements);
        setMovementDocumentCount(movementDocuments);
        setAlertCount(alerts);
        setMaintenanceCount(maintenance);
        setLifecycleCount(lifecycle);
        setStatus("Banco local conectado.");
      })
      .catch(() => {
        setStatus("Abra pelo Tauri para acessar o banco local.");
      });
  }, []);

  const stats = [
    ["Meliponários", summary.meliponaries],
    ["Espécies", summary.species],
    ["Colônias", summary.colonies],
    ["Caixas", summary.boxes],
    ["Inspeções", inspectionCount],
    ["Fotos", photoCount],
    ["Eventos", eventCount],
    ["Divisões", divisionCount],
    ["Alimentações", feedingCount],
    ["Produção", productionCount],
    ["Movimentações", movementCount],
    ["Documentos", movementDocumentCount],
    ["Manutenções", maintenanceCount],
    ["Ciclo de vida", lifecycleCount],
    ["Alertas", alertCount],
  ] as const;

  const isEmpty =
    summary.meliponaries === 0 &&
    summary.species === 0 &&
    summary.colonies === 0 &&
    summary.boxes === 0 &&
    inspectionCount === 0 &&
    photoCount === 0 &&
    eventCount === 0 &&
    divisionCount === 0 &&
    feedingCount === 0 &&
    productionCount === 0 &&
    movementCount === 0 &&
    movementDocumentCount === 0 &&
    maintenanceCount === 0 &&
    lifecycleCount === 0 &&
    alertCount === 0;

  return (
    <main className="app-shell">
      <header className="topbar">
        <div>
          <span className="eyebrow">Meliponário</span>
          <h1>MeliponarioManager</h1>
        </div>
        <span className="version">v0.1.0</span>
      </header>

      <section className="hero">
        <div>
          <p className="eyebrow">Visão geral</p>
          <h2>Seu plantel, com histórico de verdade.</h2>
          <p>
            Inspeções e suas fotos, eventos, mudanças de caixa, divisões, alimentações, produção, movimentações, documentos, manutenções e ciclo de vida preservam o contexto de cada momento.
            Alertas são derivados das pendências reais do manejo, sem duplicar estado no banco.
          </p>
        </div>
        <span className="connection-status">{status}</span>
      </section>

      <section className="stats-grid" aria-label="Resumo do meliponário">
        {stats.map(([label, value]) => (
          <article className="stat-card" key={label}>
            <span>{label}</span>
            <strong>{value}</strong>
          </article>
        ))}
      </section>

      <section className="empty-state">
        <h3>{isEmpty ? "A base do plantel está pronta" : "Dados locais carregados"}</h3>
        <p>
          O núcleo já reconhece meliponários, espécies, colônias, caixas, inspeções, fotos de inspeção, eventos, divisões, alimentações, produção,
          movimentações, documentos de rastreabilidade, manutenção de caixas, ciclo de vida e alertas derivados do manejo.
        </p>
      </section>
    </main>
  );
}

export default App;
