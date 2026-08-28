import { useEffect, useMemo, useState } from "react";
import { PageToolbar } from "../components/PageToolbar";
import { listAlerts } from "../lib/api";
import type { Alert, View } from "../types";

type Props = {
  activeMeliponaryId: string;
  onNavigate: (view: View) => void;
};

export function AlertsPage({ activeMeliponaryId, onNavigate }: Props) {
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

  const scopedItems = useMemo(
    () => activeMeliponaryId ? items.filter((item) => item.meliponaryId === activeMeliponaryId) : items,
    [activeMeliponaryId, items],
  );

  return (
    <div className="page-stack">
      <PageToolbar
        title="Alertas"
        description="Pendências derivadas do manejo e da Agenda; a ação recomendada leva ao fluxo que realmente resolve a situação."
        count={`${scopedItems.length} pendência${scopedItems.length === 1 ? "" : "s"}`}
      >
        <button type="button" className="button-secondary" onClick={() => onNavigate("agenda")}>Abrir Agenda</button>
        <button type="button" className="button-secondary" onClick={() => void reload()} disabled={loading}>Recalcular</button>
      </PageToolbar>

      <section className="panel">
        <div className="panel-heading">
          <h2>Pendências do contexto atual</h2>
          <p>Alertas de tarefa apontam para a própria Agenda; condições observadas apontam para o manejo correspondente.</p>
        </div>
        {loading ? <div className="empty-list">Calculando alertas...</div> : error ? <div className="inline-notice">{error}</div> : scopedItems.length === 0 ? (
          <div className="empty-list">Nenhuma pendência derivada dos registros atuais.</div>
        ) : (
          <div className="record-list">
            {scopedItems.map((item) => {
              const actionView = recommendedView(item.recommendedAction);
              return <article className="record-card" key={item.alertKey}>
                <div className="record-title-row">
                  <div>
                    <strong>{contextLabel(item)} · {item.title}</strong>
                    <span>{item.dueAt ? `Previsto para ${formatDateTime(item.dueAt)}` : alertTypeLabel(item.alertType)}</span>
                  </div>
                  <span className={`badge severity-${item.severity}`}>{severityLabel(item.severity)}</span>
                </div>
                {item.details && <p>{item.details}</p>}
                <div className="form-actions">
                  {item.taskId && <button type="button" onClick={() => onNavigate("agenda")}>Abrir na Agenda</button>}
                  <button type="button" className={item.taskId ? "button-secondary" : undefined} onClick={() => onNavigate(actionView)}>{recommendedLabel(item.recommendedAction)}</button>
                </div>
              </article>;
            })}
          </div>
        )}
      </section>
    </div>
  );
}

function contextLabel(item: Alert) { return item.colonyCode || item.boxCode || "Meliponário"; }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function severityLabel(value: string) { return value === "critical" ? "Crítico" : value === "attention" ? "Atenção" : "Informativo"; }
function alertTypeLabel(value: string) {
  const labels: Record<string, string> = { inspection_due: "Inspeção pendente", feeding_due: "Alimentação pendente", maintenance_due: "Manutenção pendente", weak_colony: "Colônia fraca" };
  return labels[value] || value;
}
function recommendedView(value: string): View {
  if (value === "register_feeding") return "feeding";
  if (value === "register_maintenance") return "assets";
  return "inspections";
}
function recommendedLabel(value: string) {
  if (value === "register_feeding") return "Registrar alimentação";
  if (value === "register_maintenance") return "Registrar manutenção";
  return "Registrar inspeção";
}
