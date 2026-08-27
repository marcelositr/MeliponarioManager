import { useEffect, useState } from "react";
import { listAlerts } from "../lib/api";
import type { Alert } from "../types";

export function AlertsPage() {
  const [items, setItems] = useState<Alert[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  async function reload() {
    setLoading(true);
    setError("");
    try {
      setItems(await listAlerts());
    } catch {
      setError("Não foi possível carregar os alertas derivados do manejo.");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => { void reload(); }, []);

  return (
    <div className="page-stack">
      <section className="page-heading">
        <div>
          <span className="eyebrow">Acompanhamento</span>
          <h1>Alertas</h1>
          <p>Os alertas são calculados a partir dos registros mais recentes. Para fazê-los desaparecer, registre o manejo que realmente mudou a situação.</p>
        </div>
        <span className="count-pill">{items.length} pendência{items.length === 1 ? "" : "s"}</span>
      </section>

      <section className="panel">
        <div className="panel-heading">
          <h2>Pendências do plantel</h2>
          <p>Inspeções vencidas, alimentação pendente e colônias fracas aparecem aqui sem criar um segundo estado manual.</p>
        </div>
        {loading ? <div className="empty-list">Calculando alertas...</div> : error ? <div className="inline-notice">{error}</div> : items.length === 0 ? (
          <div className="empty-list">Nenhuma pendência derivada dos registros atuais.</div>
        ) : (
          <div className="record-list">
            {items.map((item) => (
              <article className="record-card" key={item.alertKey}>
                <div className="record-title-row">
                  <div><strong>{item.colonyCode} · {item.title}</strong><span>{item.dueAt ? `Previsto para ${formatDateTime(item.dueAt)}` : alertTypeLabel(item.alertType)}</span></div>
                  <span className="badge">{severityLabel(item.severity)}</span>
                </div>
                {item.details && <p>{item.details}</p>}
              </article>
            ))}
          </div>
        )}
        <div className="form-actions" style={{ marginTop: 16 }}><button type="button" className="button-secondary" onClick={() => void reload()} disabled={loading}>Recalcular</button></div>
      </section>
    </div>
  );
}

function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function severityLabel(value: string) { return value === "critical" ? "Crítico" : value === "attention" ? "Atenção" : "Informativo"; }
function alertTypeLabel(value: string) {
  const labels: Record<string, string> = { inspection_due: "Inspeção pendente", feeding_due: "Alimentação pendente", weak_colony: "Colônia fraca" };
  return labels[value] || value;
}
