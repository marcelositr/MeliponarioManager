import { useEffect, useMemo, useState, type FormEvent } from "react";
import { listColonyLifecycle } from "../lib/api";
import type { ChangeColonyLifecycleInput, Colony, ColonyLifecycleRecord } from "../types";

type LifecyclePageProps = { colonies: Colony[]; busy: boolean; onChange: (input: ChangeColonyLifecycleInput) => Promise<boolean>; };
const initialForm: ChangeColonyLifecycleInput = { colonyId: "", action: "deactivate", occurredAt: "", reason: "", notes: "" };

export function LifecyclePage({ colonies, busy, onChange }: LifecyclePageProps) {
  const [form, setForm] = useState<ChangeColonyLifecycleInput>(initialForm);
  const [items, setItems] = useState<ColonyLifecycleRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const selectedColony = colonies.find((colony) => colony.id === form.colonyId);
  const allowedActions = useMemo(() => actionsForStatus(selectedColony?.status), [selectedColony?.status]);

  useEffect(() => {
    if (allowedActions.length > 0 && !allowedActions.some(([value]) => value === form.action)) setForm((current) => ({ ...current, action: allowedActions[0][0] }));
  }, [allowedActions, form.action]);

  useEffect(() => {
    let cancelled = false;
    if (!form.colonyId) { setItems([]); return; }
    setLoading(true);
    listColonyLifecycle(form.colonyId)
      .then((records) => { if (!cancelled) setItems(records); })
      .finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [form.colonyId]);

  async function reload(colonyId = form.colonyId) { if (colonyId) setItems(await listColonyLifecycle(colonyId)); }

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (!allowedActions.some(([value]) => value === form.action)) return;
    if (form.action === "loss" && !window.confirm("Confirmar a baixa por perda desta colônia? A operação encerra a ocupação ativa da caixa.")) return;
    const input: ChangeColonyLifecycleInput = { ...form, occurredAt: normalizeDateTime(form.occurredAt) };
    if (await onChange(input)) {
      const colonyId = form.colonyId;
      setForm({ ...initialForm, colonyId });
      await reload(colonyId);
    }
  }

  return (
    <div className="page-stack">
      <section className="page-heading"><div><span className="eyebrow">Plantel</span><h1>Ciclo de vida</h1><p>Registre baixa, inativação e reativação sem apagar a história da colônia. A entrada no plantel continua derivada do cadastro original.</p></div><span className="count-pill">{items.length} transição{items.length === 1 ? "" : "ões"}</span></section>
      <div className="content-grid">
        <section className="panel form-panel">
          <div className="panel-heading"><h2>Alterar situação</h2><p>Perda e inativação encerram a ocupação atual. Reativação não recoloca automaticamente a colônia em uma caixa.</p></div>
          <form className="form-grid" onSubmit={submit}>
            <label className="field full"><span>Colônia</span><select required value={form.colonyId} onChange={(e) => setForm({ ...form, colonyId: e.target.value })}><option value="">Selecione...</option>{colonies.map((colony) => <option value={colony.id} key={colony.id}>{colony.code} · {statusLabel(colony.status)}</option>)}</select></label>
            {selectedColony && allowedActions.length === 0 ? <div className="inline-notice field full">Esta situação não permite nova transição pela rotina de ciclo de vida.</div> : <>
              <label className="field"><span>Ação</span><select value={form.action} onChange={(e) => setForm({ ...form, action: e.target.value })}>{allowedActions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label>
              <label className="field"><span>Data e hora</span><input type="datetime-local" value={form.occurredAt} onChange={(e) => setForm({ ...form, occurredAt: e.target.value })} /></label>
              <label className="field full"><span>Motivo</span><input value={form.reason} onChange={(e) => setForm({ ...form, reason: e.target.value })} placeholder={form.action === "loss" ? "Ex.: abandono, morte, ataque..." : "Motivo da alteração"} /></label>
              <label className="field full"><span>Observações</span><textarea rows={3} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} /></label>
              {form.action === "loss" && <div className="inline-notice field full">Baixa por perda é um fato histórico terminal. A colônia deixa de ocupar a caixa e não pode ser reativada por esta rotina.</div>}
              {form.action === "reactivate" && <div className="inline-notice field full">Reativar devolve a situação para ativa, mas a caixa deve ser atribuída separadamente.</div>}
              <div className="form-actions full"><button type="submit" disabled={busy || !form.colonyId || allowedActions.length === 0}>{busy ? "Salvando..." : actionButton(form.action)}</button></div>
            </>}
          </form>
        </section>

        <section className="panel list-panel">
          <div className="panel-heading"><h2>Histórico de transições</h2><p>O registro preserva situação anterior, nova situação, caixa da data e motivo informado.</p></div>
          {!form.colonyId ? <div className="empty-list">Selecione uma colônia para consultar o histórico.</div> : loading ? <div className="empty-list">Carregando histórico...</div> : items.length === 0 ? <div className="empty-list">Nenhuma transição registrada. A entrada inicial continua visível na timeline da colônia.</div> : <div className="record-list">{items.map((item) => <article className="record-card" key={item.id}><div className="record-title-row"><div><strong>{actionLabel(item.action)}</strong><span>{formatDateTime(item.occurredAt)} · {item.boxCode ? `Caixa ${item.boxCode}` : "sem caixa associada"}</span></div><span className="badge">{statusLabel(item.previousStatus)} → {statusLabel(item.newStatus)}</span></div>{item.reason && <p><strong>Motivo:</strong> {item.reason}</p>}{item.notes && <p>{item.notes}</p>}</article>)}</div>}
        </section>
      </div>
    </div>
  );
}

function actionsForStatus(status?: string): Array<[string, string]> {
  if (status === "inactive") return [["reactivate", "Reativar colônia"]];
  if (status === "active" || status === "weak" || status === "recovering") return [["deactivate", "Inativar colônia"], ["loss", "Baixa por perda"]];
  return [];
}
function normalizeDateTime(value?: string) { if (!value) return undefined; const normalized = value.replace("T", " "); return normalized.length === 16 ? `${normalized}:00` : normalized; }
function formatDateTime(value: string) { return value.replace("T", " ").slice(0, 16); }
function actionLabel(value: string) { return value === "loss" ? "Baixa por perda" : value === "deactivate" ? "Colônia inativada" : "Colônia reativada"; }
function actionButton(value?: string) { return value === "loss" ? "Registrar baixa" : value === "reactivate" ? "Reativar colônia" : "Inativar colônia"; }
function statusLabel(value?: string) { const labels: Record<string, string> = { active: "Ativa", weak: "Fraca", recovering: "Em recuperação", inactive: "Inativa", lost: "Perdida", transferred: "Transferida" }; return value ? labels[value] || value : ""; }
