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
  const [eventCount, setEventCount] = useState(0);
  const [status, setStatus] = useState("Carregando dados locais...");

  useEffect(() => {
    Promise.all([
      invoke<CoreSummary>("get_core_summary"),
      invoke<number>("get_inspection_count"),
      invoke<number>("get_event_count"),
    ])
      .then(([data, inspections, events]) => {
        setSummary(data);
        setInspectionCount(inspections);
        setEventCount(events);
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
    ["Eventos", eventCount],
  ] as const;

  const isEmpty =
    summary.meliponaries === 0 &&
    summary.species === 0 &&
    summary.colonies === 0 &&
    summary.boxes === 0 &&
    inspectionCount === 0 &&
    eventCount === 0;

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
            Inspeções, eventos e mudanças de caixa preservam o contexto de cada momento.
            O histórico da colônia é montado a partir desses fatos, sem apagar o passado.
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
          O núcleo já reconhece meliponários, espécies, colônias, caixas, inspeções e eventos.
          A linha do tempo reúne esses registros sem duplicar a fonte de verdade.
        </p>
      </section>
    </main>
  );
}

export default App;
