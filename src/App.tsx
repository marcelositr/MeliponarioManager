const stats = [
  ["Colônias", "0"],
  ["Fortes", "0"],
  ["Em atenção", "0"],
  ["Produzindo", "0"],
];

function App() {
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
            Acompanhe colônias, caixas, inspeções, manejo e movimentações sem perder a origem de cada registro.
          </p>
        </div>
        <button type="button">Adicionar colônia</button>
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
        <h3>Comece pelo cadastro do seu meliponário</h3>
        <p>
          Esta é a base inicial da aplicação. Os módulos de colônias, caixas e inspeções entram nas próximas etapas da série 0.x.
        </p>
      </section>
    </main>
  );
}

export default App;
