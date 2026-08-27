import { useEffect, useState, type FormEvent } from "react";
import { listColonyInspections } from "../lib/api";
import type { Colony, CreateInspectionInput, Inspection } from "../types";

type InspectionsPageProps = {
  colonies: Colony[];
  busy: boolean;
  onCreate: (input: CreateInspectionInput) => Promise<boolean>;
};

const initialForm: CreateInspectionInput = {
  colonyId: "",
  inspectedAt: "",
  strength: "unknown",
  layingStatus: "",
  foodReserves: "",
  broodStatus: "",
  pestsNotes: "",
  observations: "",
  actionsTaken: "",
  nextInspectionAt: "",
};

const strengthOptions = [
  ["strong", "Forte"],
  ["medium", "Média"],
  ["weak", "Fraca"],
  ["unknown", "Não avaliada"],
] as const;

export function InspectionsPage({ colonies, busy, onCreate }: InspectionsPageProps) {
  const [form, setForm] = useState<CreateInspectionInput>(initialForm);
  const [queenValue, setQueenValue] = useState("");
  const [items, setItems] = useState<Inspection[]>([]);
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
    listColonyInspections(form.colonyId)
      .then((data) => {
        if (!cancelled) setItems(data);
      })
      .catch(() => {
        if (!cancelled) setLoadError("Não foi possível carregar as inspeções desta colônia.");
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [form.colonyId]);

  async function reload() {
    if (!form.colonyId) return;
    setItems(await listColonyInspections(form.colonyId));
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    const input: CreateInspectionInput = {
      ...form,
      inspectedAt: normalizeDateTime(form.inspectedAt),
      nextInspectionAt: normalizeDateTime(form.nextInspectionAt),
      queenPresent: queenValue === "yes" ? true : queenValue === "no" ? false : null,
    };

    if (await onCreate(input)) {
      const colonyId = form.colonyId;
      setForm({ ...initialForm, colonyId });
      setQueenValue("");
      await reload();
    }
  }

  return (
    <div className="page-stack">
      <section className="page-heading">
        <div>
          <span className="eyebrow">Manejo</span>
          <h1>Inspeções</h1>
          <p>Registre a condição observada da colônia. A caixa exibida no histórico é resolvida pelo backend conforme a data da inspeção.</p>
        </div>
        <span className="count-pill">{items.length} no histórico selecionado</span>
      </section>

      <div className="content-grid">
        <section className="panel form-panel">
          <div className="panel-heading">
            <h2>Nova inspeção</h2>
            <p>Os campos de manejo são descritivos; a força usa uma classificação estável para permitir alertas.</p>
          </div>

          {colonies.length === 0 ? (
            <div className="inline-notice">Cadastre uma colônia antes de registrar inspeções.</div>
          ) : (
            <form className="form-grid" onSubmit={submit}>
              <label className="field full">
                <span>Colônia</span>
                <select required value={form.colonyId} onChange={(e) => setForm({ ...form, colonyId: e.target.value })}>
                  <option value="">Selecione...</option>
                  {colonies.map((colony) => (
                    <option value={colony.id} key={colony.id}>{colony.code} {colony.currentBoxCode ? `· ${colony.currentBoxCode}` : "· sem caixa"}</option>
                  ))}
                </select>
              </label>

              <label className="field">
                <span>Data e hora</span>
                <input type="datetime-local" value={form.inspectedAt} onChange={(e) => setForm({ ...form, inspectedAt: e.target.value })} />
              </label>
              <label className="field">
                <span>Força</span>
                <select value={form.strength} onChange={(e) => setForm({ ...form, strength: e.target.value })}>
                  {strengthOptions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}
                </select>
              </label>

              <label className="field">
                <span>Rainha presente</span>
                <select value={queenValue} onChange={(e) => setQueenValue(e.target.value)}>
                  <option value="">Não avaliado</option>
                  <option value="yes">Sim</option>
                  <option value="no">Não</option>
                </select>
              </label>
              <label className="field">
                <span>Postura</span>
                <input value={form.layingStatus} onChange={(e) => setForm({ ...form, layingStatus: e.target.value })} placeholder="Ex.: regular, intensa, ausente" />
              </label>
              <label className="field">
                <span>Reservas de alimento</span>
                <input value={form.foodReserves} onChange={(e) => setForm({ ...form, foodReserves: e.target.value })} placeholder="Ex.: boas, baixas" />
              </label>
              <label className="field">
                <span>Condição das crias</span>
                <input value={form.broodStatus} onChange={(e) => setForm({ ...form, broodStatus: e.target.value })} placeholder="Ex.: normal" />
              </label>
              <label className="field full">
                <span>Pragas ou inimigos</span>
                <input value={form.pestsNotes} onChange={(e) => setForm({ ...form, pestsNotes: e.target.value })} />
              </label>
              <label className="field full">
                <span>Observações</span>
                <textarea rows={3} value={form.observations} onChange={(e) => setForm({ ...form, observations: e.target.value })} />
              </label>
              <label className="field full">
                <span>Ações realizadas</span>
                <textarea rows={2} value={form.actionsTaken} onChange={(e) => setForm({ ...form, actionsTaken: e.target.value })} />
              </label>
              <label className="field full">
                <span>Próxima inspeção</span>
                <input type="datetime-local" value={form.nextInspectionAt} onChange={(e) => setForm({ ...form, nextInspectionAt: e.target.value })} />
              </label>
              <div className="form-actions full">
                <button disabled={busy || !form.colonyId} type="submit">{busy ? "Salvando..." : "Registrar inspeção"}</button>
              </div>
            </form>
          )}
        </section>

        <section className="panel list-panel">
          <div className="panel-heading">
            <h2>Histórico da colônia</h2>
            <p>Selecione uma colônia no formulário para consultar as inspeções mais recentes primeiro.</p>
          </div>
          {!form.colonyId ? (
            <div className="empty-list">Selecione uma colônia para consultar o histórico.</div>
          ) : loading ? (
            <div className="empty-list">Carregando inspeções...</div>
          ) : loadError ? (
            <div className="inline-notice">{loadError}</div>
          ) : items.length === 0 ? (
            <div className="empty-list">Nenhuma inspeção registrada para esta colônia.</div>
          ) : (
            <div className="record-list">
              {items.map((item) => (
                <article className="record-card" key={item.id}>
                  <div className="record-title-row">
                    <div>
                      <strong>{formatDateTime(item.inspectedAt)}</strong>
                      <span>{item.boxCode ? `Caixa ${item.boxCode}` : "Sem caixa na data"}</span>
                    </div>
                    <span className={`badge status-${item.strength}`}>{strengthLabel(item.strength)}</span>
                  </div>
                  <dl>
                    <div><dt>Rainha</dt><dd>{booleanLabel(item.queenPresent)}</dd></div>
                    <div><dt>Postura</dt><dd>{item.layingStatus || "Não informado"}</dd></div>
                    <div><dt>Reservas</dt><dd>{item.foodReserves || "Não informado"}</dd></div>
                    <div><dt>Crias</dt><dd>{item.broodStatus || "Não informado"}</dd></div>
                    <div><dt>Próxima</dt><dd>{item.nextInspectionAt ? formatDateTime(item.nextInspectionAt) : "Sem agendamento"}</dd></div>
                    <div><dt>Pragas</dt><dd>{item.pestsNotes || "Nenhum registro"}</dd></div>
                  </dl>
                  {item.observations && <p>{item.observations}</p>}
                  {item.actionsTaken && <p><strong>Ações:</strong> {item.actionsTaken}</p>}
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

function formatDateTime(value: string) {
  return value.replace("T", " ").slice(0, 16);
}

function strengthLabel(value: string) {
  return strengthOptions.find(([key]) => key === value)?.[1] || value;
}

function booleanLabel(value?: boolean | null) {
  if (value === true) return "Presente";
  if (value === false) return "Não observada";
  return "Não avaliado";
}
