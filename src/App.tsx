import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type AppStatus = {
  app_name: string;
  version: string;
  database: string;
};

function App() {
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<AppStatus>("app_status")
      .then(setStatus)
      .catch((reason) => setError(String(reason)));
  }, []);

  return (
    <main className="app-shell">
      <header className="topbar">
        <div>
          <p className="eyebrow">MeliponarioManager</p>
          <h1>Seu meliponário, com histórico de verdade.</h1>
          <p className="subtitle">
            Controle de colônias, caixas, inspeções e movimentações sem apagar o passado.
          </p>
        </div>
        <span className="version">v0.1.0-dev</span>
      </header>

      <section className="status-card">
        <h2>Estado da aplicação</h2>
        {status && (
          <dl>
            <div>
              <dt>Aplicação</dt>
              <dd>{status.app_name}</dd>
            </div>
            <div>
              <dt>Versão</dt>
              <dd>{status.version}</dd>
            </div>
            <div>
              <dt>Banco local</dt>
              <dd>{status.database}</dd>
            </div>
          </dl>
        )}
        {!status && !error && <p>Inicializando banco local…</p>}
        {error && <p className="error">Falha ao iniciar: {error}</p>}
      </section>

      <section className="grid">
        <article>
          <span>01</span>
          <h3>Meliponários</h3>
          <p>Unidades de criação e localização do plantel.</p>
        </article>
        <article>
          <span>02</span>
          <h3>Colônias</h3>
          <p>Identidade própria, espécie, origem e situação atual.</p>
        </article>
        <article>
          <span>03</span>
          <h3>Inspeções</h3>
          <p>Força, cria, postura, rainha, alimento, pragas e observações.</p>
        </article>
        <article>
          <span>04</span>
          <h3>Histórico</h3>
          <p>Eventos preservados para rastrear o que aconteceu com cada colônia.</p>
        </article>
      </section>
    </main>
  );
}

export default App;
