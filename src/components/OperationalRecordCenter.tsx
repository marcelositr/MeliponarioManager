import { useEffect, useState } from "react";
import { listAlerts } from "../lib/api";
import {
  getBoxRecordCenter,
  getColonyRecordCenter,
  getMeliponaryRecordCenter,
  listBoxContextPhotos,
  listBoxOccupancies,
  listTasks,
} from "../lib/agenda-api";
import type {
  BoxOccupancyHistory,
  BoxRecordCenter,
  ColonyRecordCenter,
  MeliponaryRecordCenter,
  ScheduledTask,
} from "../lib/agenda-types";
import type { Navigate, NavigationIntent } from "../lib/navigation";
import { formatDateTimeBr } from "../lib/presentation";
import type { Alert, InspectionPhoto, View } from "../types";

type NavigateProps = { onNavigate: (view: View) => void };

export function ColonyOperationalCenter({ colonyId, onNavigate }: NavigateProps & { colonyId: string }) {
  const navigate = onNavigate as Navigate;
  const [center, setCenter] = useState<ColonyRecordCenter | null>(null);
  const [tasks, setTasks] = useState<ScheduledTask[]>([]);
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [error, setError] = useState("");

  useEffect(() => {
    let cancelled = false;
    setError("");
    Promise.all([
      getColonyRecordCenter(colonyId),
      listTasks({ view: "pending", colonyId }),
      listAlerts(),
    ]).then(([nextCenter, nextTasks, nextAlerts]) => {
      if (cancelled) return;
      setCenter(nextCenter);
      setTasks(nextTasks);
      setAlerts(nextAlerts.filter((item) => item.colonyId === colonyId));
    }).catch(() => {
      if (!cancelled) setError("Não foi possível carregar o centro operacional desta colônia.");
    });
    return () => { cancelled = true; };
  }, [colonyId]);

  if (error) return <div className="inline-notice" role="alert">{error}</div>;
  if (!center) return <div className="empty-list" role="status">Carregando ficha operacional...</div>;

  return <div className="page-stack compact-stack">
    <section className="panel">
      <div className="panel-heading"><h2>Centro operacional</h2><p>Resumo derivado dos registros atuais; cada fato continua pertencendo ao seu fluxo próprio.</p></div>
      <div className="summary-grid">
        <Metric label="Situação" value={statusLabel(center.status)} />
        <Metric label="Caixa atual" value={center.currentBoxCode || "Sem caixa"} />
        <Metric label="Última inspeção" value={center.latestInspectionAt ? formatDateTimeBr(center.latestInspectionAt) : "Sem inspeção"} />
        <Metric label="Força observada" value={strengthLabel(center.latestStrength)} />
        <Metric label="Última alimentação" value={center.latestFeedingAt ? formatDateTimeBr(center.latestFeedingAt) : "Sem alimentação"} />
        <Metric label="Agenda" value={`${center.pendingTasks} pendente${center.pendingTasks === 1 ? "" : "s"} · ${center.overdueTasks} atrasada${center.overdueTasks === 1 ? "" : "s"}`} />
        <Metric label="Alertas atuais" value={String(center.currentAlerts)} />
        <Metric label="Próximo compromisso" value={center.nextTaskTitle ? `${center.nextTaskTitle}${center.nextTaskAt ? ` · ${formatDateTimeBr(center.nextTaskAt)}` : ""}` : "Nenhum"} />
      </div>
      <div className="workspace-actions">
        <button type="button" onClick={() => navigate({ view: "agenda", colonyId })}>Abrir Agenda</button>
        <button className="button-secondary" type="button" onClick={() => navigate({ view: "inspections", colonyId })}>Inspeções</button>
        <button className="button-secondary" type="button" onClick={() => navigate({ view: "feeding", colonyId })}>Alimentação</button>
        <button className="button-secondary" type="button" onClick={() => navigate({ view: "history", colonyId })}>Histórico</button>
        <button className="button-secondary" type="button" onClick={() => navigate({ view: "movements", colonyId })}>Movimentações</button>
      </div>
    </section>
    <TaskPanel tasks={tasks} onNavigate={navigate} />
    <AlertPanel alerts={alerts} onNavigate={navigate} />
  </div>;
}

export function BoxOperationalCenter({ boxId, onNavigate }: NavigateProps & { boxId: string }) {
  const navigate = onNavigate as Navigate;
  const [center, setCenter] = useState<BoxRecordCenter | null>(null);
  const [tasks, setTasks] = useState<ScheduledTask[]>([]);
  const [occupancies, setOccupancies] = useState<BoxOccupancyHistory[]>([]);
  const [photos, setPhotos] = useState<InspectionPhoto[]>([]);
  const [error, setError] = useState("");

  useEffect(() => {
    let cancelled = false;
    setError("");
    Promise.all([
      getBoxRecordCenter(boxId),
      listTasks({ view: "pending", boxId }),
      listBoxOccupancies(boxId),
      listBoxContextPhotos(boxId),
    ]).then(([nextCenter, nextTasks, nextOccupancies, nextPhotos]) => {
      if (cancelled) return;
      setCenter(nextCenter);
      setTasks(nextTasks);
      setOccupancies(nextOccupancies);
      setPhotos(nextPhotos);
    }).catch(() => {
      if (!cancelled) setError("Não foi possível carregar o centro operacional desta caixa.");
    });
    return () => { cancelled = true; };
  }, [boxId]);

  if (error) return <div className="inline-notice" role="alert">{error}</div>;
  if (!center) return <div className="empty-list" role="status">Carregando ficha operacional...</div>;

  return <div className="page-stack compact-stack">
    <section className="panel">
      <div className="panel-heading"><h2>Centro operacional</h2><p>A caixa mantém identidade física própria, separada das colônias que a ocuparam.</p></div>
      <div className="summary-grid">
        <Metric label="Estado físico" value={boxStatusLabel(center.status)} />
        <Metric label="Colônia atual" value={center.currentColonyCode || "Livre"} />
        <Metric label="Ocupações registradas" value={String(center.occupancyRecords)} />
        <Metric label="Manutenções" value={String(center.maintenanceRecords)} />
        <Metric label="Tarefas pendentes" value={String(center.pendingTasks)} />
        <Metric label="Próxima manutenção" value={center.nextMaintenanceAt ? formatDateTimeBr(center.nextMaintenanceAt) : "Não agendada"} />
        <Metric label="Fotos no contexto da caixa" value={String(photos.length)} />
      </div>
      <div className="workspace-actions">
        <button type="button" onClick={() => navigate({ view: "agenda", boxId })}>Abrir Agenda</button>
        <button className="button-secondary" type="button" onClick={() => navigate({ view: "assets", boxId })}>Manutenção</button>
        <button className="button-secondary" type="button" onClick={() => navigate({ view: "boxes", boxId })}>Esta caixa</button>
      </div>
    </section>
    <TaskPanel tasks={tasks} onNavigate={navigate} />
    <section className="panel wide-list">
      <div className="panel-heading"><h2>Histórico de ocupação</h2><p>Intervalos preservados de colônia ↔ caixa.</p></div>
      {occupancies.length === 0 ? <div className="empty-list">Esta caixa ainda não possui ocupações registradas.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Colônia</th><th>Início</th><th>Fim</th><th>Motivo</th><th>Estado</th></tr></thead><tbody>{occupancies.map((item) => <tr key={item.id}><td><strong>{item.colonyCode}</strong></td><td>{formatDateTimeBr(item.startedAt)}</td><td>{item.endedAt ? formatDateTimeBr(item.endedAt) : "Atual"}</td><td>{item.reason || "—"}</td><td>{item.correctedAt ? "Corrigido" : "Original"}</td></tr>)}</tbody></table></div>}
    </section>
    <section className="panel">
      <div className="panel-heading"><h2>Fotos no contexto histórico</h2><p>Fotos pertencem às inspeções; esta ficha apenas projeta aquelas feitas enquanto a caixa era o contexto físico.</p></div>
      {photos.length === 0 ? <div className="empty-list">Nenhuma foto associada ao contexto histórico desta caixa.</div> : <div className="record-list">{photos.slice(0, 6).map((photo) => <article className="record-card" key={photo.id}><div className="record-title-row"><div><strong>{photo.originalName}</strong><span>{photo.colonyCode} · {formatDateTimeBr(photo.capturedAt)}</span></div></div>{photo.notes && <p>{photo.notes}</p>}</article>)}</div>}
    </section>
  </div>;
}

export function MeliponaryOperationalCenter({ meliponaryId, onNavigate }: NavigateProps & { meliponaryId: string }) {
  const navigate = onNavigate as Navigate;
  const [center, setCenter] = useState<MeliponaryRecordCenter | null>(null);
  const [tasks, setTasks] = useState<ScheduledTask[]>([]);
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [error, setError] = useState("");

  useEffect(() => {
    let cancelled = false;
    setError("");
    Promise.all([
      getMeliponaryRecordCenter(meliponaryId),
      listTasks({ view: "pending", meliponaryId }),
      listAlerts(),
    ]).then(([nextCenter, nextTasks, nextAlerts]) => {
      if (cancelled) return;
      setCenter(nextCenter);
      setTasks(nextTasks);
      setAlerts(nextAlerts.filter((item) => item.meliponaryId === meliponaryId));
    }).catch(() => {
      if (!cancelled) setError("Não foi possível carregar o centro operacional deste meliponário.");
    });
    return () => { cancelled = true; };
  }, [meliponaryId]);

  if (error) return <div className="inline-notice" role="alert">{error}</div>;
  if (!center) return <div className="empty-list" role="status">Carregando ficha operacional...</div>;

  return <div className="page-stack compact-stack">
    <section className="panel">
      <div className="panel-heading"><h2>Centro operacional</h2><p>Leitura consolidada da unidade, sem criar um segundo estado para plantel, Agenda ou alertas.</p></div>
      <div className="summary-grid">
        <Metric label="Colônias" value={String(center.colonies)} />
        <Metric label="Caixas" value={String(center.boxes)} />
        <Metric label="Tarefas pendentes" value={String(center.pendingTasks)} />
        <Metric label="Tarefas atrasadas" value={String(center.overdueTasks)} />
        <Metric label="Alertas atuais" value={String(center.alerts)} />
        <Metric label="Produções recentes" value={String(center.recentProductionRecords)} />
      </div>
      <div className="workspace-actions">
        <button type="button" onClick={() => navigate({ view: "agenda", meliponaryId })}>Abrir Agenda</button>
        <button className="button-secondary" type="button" onClick={() => navigate({ view: "alerts", meliponaryId })}>Alertas</button>
        <button className="button-secondary" type="button" onClick={() => navigate({ view: "colonies", meliponaryId })}>Colônias</button>
        <button className="button-secondary" type="button" onClick={() => navigate({ view: "boxes", meliponaryId })}>Caixas</button>
      </div>
    </section>
    <TaskPanel tasks={tasks} onNavigate={navigate} />
    <AlertPanel alerts={alerts} onNavigate={navigate} />
  </div>;
}

function TaskPanel({ tasks, onNavigate }: { tasks: ScheduledTask[]; onNavigate: Navigate }) {
  return <section className="panel wide-list">
    <div className="panel-heading"><h2>Agenda pendente</h2><p>Compromissos são planos operacionais; fatos realizados permanecem nos registros próprios.</p></div>
    {tasks.length === 0 ? <div className="empty-list">Nenhum compromisso pendente neste contexto.</div> : <div className="table-wrap"><table className="data-table"><thead><tr><th>Quando</th><th>Tarefa</th><th>Tipo</th><th>Prioridade</th><th>Ação</th></tr></thead><tbody>{tasks.slice(0, 6).map((task) => <tr key={task.id} className={task.overdue ? "attention-row" : undefined}><td>{formatDateTimeBr(task.scheduledFor)}</td><td><strong>{task.title}</strong></td><td>{taskTypeLabel(task.taskType)}</td><td>{priorityLabel(task.priority)}</td><td><button className="button-secondary" type="button" onClick={() => onNavigate({ view: "agenda", taskId: task.id, colonyId: task.colonyId, boxId: task.boxId, meliponaryId: task.meliponaryId })}>Abrir</button></td></tr>)}</tbody></table></div>}
    <div className="form-actions"><button className="button-secondary" type="button" onClick={() => onNavigate(tasks[0] ? { view: "agenda", colonyId: tasks[0].colonyId, boxId: tasks[0].boxId, meliponaryId: tasks[0].meliponaryId } : "agenda")}>Gerenciar na Agenda</button></div>
  </section>;
}

function AlertPanel({ alerts, onNavigate }: { alerts: Alert[]; onNavigate: Navigate }) {
  return <section className="panel">
    <div className="panel-heading"><h2>Alertas atuais</h2><p>Pendências derivadas do estado operacional e dos compromissos vencidos.</p></div>
    {alerts.length === 0 ? <div className="empty-list">Nenhum alerta atual neste contexto.</div> : <div className="record-list">{alerts.slice(0, 5).map((alert) => <article className="record-card" key={alert.alertKey}><div className="record-title-row"><div><strong>{alert.title}</strong><span>{alert.dueAt ? formatDateTimeBr(alert.dueAt) : alertTypeLabel(alert.alertType)}</span></div><span className={`badge severity-${alert.severity}`}>{severityLabel(alert.severity)}</span></div>{alert.details && <p>{alert.details}</p>}<div className="form-actions"><button className="button-secondary" type="button" onClick={() => onNavigate(alert.taskId ? { view: "agenda", taskId: alert.taskId, colonyId: alert.colonyId, boxId: alert.boxId, meliponaryId: alert.meliponaryId } : recommendedIntent(alert))}>{alert.taskId ? "Abrir tarefa" : "Resolver"}</button></div></article>)}</div>}
    <div className="form-actions"><button className="button-secondary" type="button" onClick={() => onNavigate("alerts")}>Abrir central de alertas</button></div>
  </section>;
}

function Metric({ label, value }: { label: string; value: string }) {
  return <div className="summary-item"><span>{label}</span><strong>{value}</strong></div>;
}

function recommendedIntent(alert: Alert): NavigationIntent {
  const view: View = alert.recommendedAction === "register_feeding" ? "feeding" : alert.recommendedAction === "register_maintenance" ? "assets" : "inspections";
  return { view, colonyId: alert.colonyId, boxId: alert.boxId, meliponaryId: alert.meliponaryId, action: "create" };
}
function statusLabel(value: string) { const labels: Record<string, string> = { active: "Ativa", weak: "Fraca (legado)", recovering: "Em recuperação (legado)", inactive: "Inativa", lost: "Perdida", transferred: "Transferida" }; return labels[value] || value; }
function boxStatusLabel(value: string) { const labels: Record<string, string> = { active: "Ativa", maintenance: "Manutenção", retired: "Aposentada" }; return labels[value] || value; }
function strengthLabel(value?: string | null) { const labels: Record<string, string> = { strong: "Forte", medium: "Média", weak: "Fraca", unknown: "Sem avaliação" }; return value ? labels[value] || value : "Sem avaliação"; }
function taskTypeLabel(value: string) { const labels: Record<string, string> = { inspection: "Inspeção", feeding: "Alimentação", maintenance: "Manutenção", generic: "Geral" }; return labels[value] || value; }
function priorityLabel(value: string) { return value === "critical" ? "Crítica" : value === "attention" ? "Atenção" : "Normal"; }
function severityLabel(value: string) { return value === "critical" ? "Crítico" : value === "attention" ? "Atenção" : "Informativo"; }
function alertTypeLabel(value: string) { const labels: Record<string, string> = { inspection_due: "Inspeção pendente", feeding_due: "Alimentação pendente", maintenance_due: "Manutenção pendente", weak_colony: "Colônia fraca" }; return labels[value] || value; }
