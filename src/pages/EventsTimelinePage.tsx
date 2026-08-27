import { useEffect, useState, type FormEvent } from "react";
import { getColonyTimeline } from "../lib/api";
import type { Colony, CreateColonyEventInput, TimelineEntry } from "../types";

type EventsTimelinePageProps = {
  colonies: Colony[];
  busy: boolean;
  onCreate: (input: CreateColonyEventInput) => Promise<boolean>;
};

const initialForm: CreateColonyEventInput = {
  colonyId: "",
  eventType: "observation",
  occurredAt: "",
  title: "",
  details: "",
  severity: "info",
};

const eventTypes = [
  ["swarming", "Enxameação"],
  ["abandonment", "Abandono"],
  ["queen_loss", "Perda de rainha"],
  ["attack", "Ataque"],
  ["pest", "Praga ou inimigo"],
  ["recovery", "Recuperação"],
  ["maintenance", "Manutenção da colônia"],
  ["observation", "Observação"],
  ["other", "Outro evento"],
] as const;

export function EventsTimelinePage({ colonies, busy, onCreate }: EventsTimelinePageProps) {
  const [form, setForm] = useState<CreateColonyEventInput>(initialForm);
  const [items, setItems] = useState<TimelineEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadError, setLoadError] = useState("");

  useEffect(() => {
    let cancelled = false;
    if (!form.colonyId) {
      setItems([]);
      setLoadError("");
      return;
    }
    setLoading(true);
    setLoadError("");
    getColonyTimeline(form.colonyId)
      .then((data) => { if (!cancelled) setItems(data); })
      .catch(() => { if (!cancelled) setLoadError("Não foi possível carregar a timeline desta colônia."); })
      .finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [form.colonyId]);

  async function reload(colonyId = form.colonyId) {
    if (!colonyId) return;
    setItems(await getColonyTimeline(colonyId));
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    const input: CreateColonyEventInput = {
      ...form,
      occurredAt: normalizeDateTime(form.occurredAt),
    };
    if (await onCreate(input)) {
      const colonyId = form.colonyId;
      setForm({ ...initialForm, colonyId });
      await reload(colonyId);
    }
  }

  return (
    <div className="page-stack">
      <section className="page-heading">
        <div>
          <span className="eyebrow">Rastreabilidade</span>
          <h1>Eventos e timeline</h1>
          <p>Registre acontecimentos pontuais e acompanhe, numa única sequência, todos os fatos históricos da colônia.</p>
        </div>
        <span className="count-pill">{items.length} fatos na timeline</span>
      </section>

      <div className="content-grid">
        <section className="panel form-panel">
          <div className="panel-heading">
            <h2>Novo evento</h2>
            <p>Use eventos para fatos que não pertencem a uma inspeção, alimentação, produção ou movimentação específica.</p>
          </div>
          {colonies.length === 0 ? (
            <div className="inline-notice">Cadastre uma colônia antes de registrar eventos.</div>
          ) : (
            <form className="form-grid" onSubmit={submit}>
              <label className="field full"><span>Colônia</span><select required value={form.colonyId} onChange={(e) => setForm({ ...form, colonyId: e.target.value })}><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>)}</select></label>
              <label className="field"><span>Tipo</span><select value={form.eventType} onChange={(e) => setForm({ ...form, eventType: e.target.value })}>{eventTypes.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label>
              <label className="field"><span>Importância</span><select value={form.severity} onChange={(e) => setForm({ ...form, severity: e.target.value })}><option value="info">Informativo</option><option value="attention">Atenção</option><option value="critical">Crítico</option></select></label>
              <label className="field full"><span>Data e hora</span><input type="datetime-local" value={form.occurredAt} onChange={(e) => setForm({ ...form, occurredAt: e.target.value })} /></label>
              <label className="field full"><span>Título opcional</span><input value={form.title} onChange={(e) => setForm({ ...form, title: e.target.value })} placeholder="Se vazio, o sistema usa o nome do tipo de evento" /></label>
              <label className="field full"><span>Detalhes</span><textarea rows={4} value={form.details} onChange={(e) => setForm({ ...form, details: e.target.value })} /></label>
              <div className="form-actions full"><button disabled={busy || !form.colonyId} type="submit">{busy ? "Salvando..." : "Registrar evento"}</button></div>
            </form>
          )}
        </section>

        <section className="panel list-panel">
          <div className="panel-heading">
            <h2>Histórico unificado</h2>
            <p>Inspeções, alimentação, produção, movimentações, ocupações, manutenção, ciclo de vida e eventos aparecem em ordem cronológica.</p>
          </div>
          {!form.colonyId ? <div className="empty-list">Selecione uma colônia para consultar a timeline.</div> : loading ? <div className="empty-list">Carregando histórico...</div> : loadError ? <div className="inline-notice">{loadError}</div> : items.length === 0 ? <div className="empty-list">Nenhum fato histórico encontrado.</div> : (
            <div className="record-list">
              {items.map((item) => (
                <article className="record-card" key={`${item.sourceType}-${item.sourceId}`}>
                  <div className="record-title-row">
                    <div><strong>{item.title}</strong><span>{formatDateTime(item.occurredAt)} · {item.boxCode ? `Caixa ${item.boxCode}` : "sem caixa associada"}</span></div>
                    <span className="badge">{sourceLabel(item.sourceType)} · {severityLabel(item.severity)}</span>
                  </div>
                  {item.details && <p>{item.details}</p>}
                </article>
              ))}
            </div>
          )}
        </section>
      </div>
    </div>
  );
}

function normalizeDateTime(value?: string) {
  if (!value) return undefined;
  const normalized = value.replace("T", " ");
  return normalized.length === 16 ? `${normalized}:00` : normalized;
}
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function severityLabel(value: string) { return value === "critical" ? "Crítico" : value === "attention" ? "Atenção" : "Info"; }
function sourceLabel(value: string) {
  const labels: Record<string, string> = { event: "Evento", inspection: "Inspeção", feeding: "Alimentação", production: "Produção", movement: "Movimentação", box_occupancy: "Caixa", box_maintenance: "Manutenção", lifecycle: "Ciclo de vida" };
  return labels[value] || value;
}
