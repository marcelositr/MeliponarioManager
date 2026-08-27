import { useEffect, useState, type FormEvent } from "react";
import { listColonyFeedings } from "../lib/api";
import type { Colony, CreateFeedingInput, Feeding } from "../types";

type FeedingPageProps = {
  colonies: Colony[];
  busy: boolean;
  onCreate: (input: CreateFeedingInput) => Promise<boolean>;
};

type FeedingForm = {
  colonyId: string;
  fedAt: string;
  foodType: string;
  quantity: string;
  unit: string;
  responseNotes: string;
  notes: string;
  nextFeedingAt: string;
};

const initialForm: FeedingForm = {
  colonyId: "",
  fedAt: "",
  foodType: "",
  quantity: "",
  unit: "",
  responseNotes: "",
  notes: "",
  nextFeedingAt: "",
};

export function FeedingPage({ colonies, busy, onCreate }: FeedingPageProps) {
  const [form, setForm] = useState<FeedingForm>(initialForm);
  const [items, setItems] = useState<Feeding[]>([]);
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
    listColonyFeedings(form.colonyId)
      .then((data) => {
        if (!cancelled) setItems(data);
      })
      .catch(() => {
        if (!cancelled) setLoadError("Não foi possível carregar as alimentações desta colônia.");
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [form.colonyId]);

  async function reload(colonyId = form.colonyId) {
    if (!colonyId) return;
    setItems(await listColonyFeedings(colonyId));
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    const quantity = form.quantity.trim() ? Number(form.quantity) : undefined;
    const input: CreateFeedingInput = {
      colonyId: form.colonyId,
      fedAt: normalizeDateTime(form.fedAt),
      foodType: form.foodType,
      quantity,
      unit: form.unit.trim() || undefined,
      responseNotes: form.responseNotes,
      notes: form.notes,
      nextFeedingAt: normalizeDateTime(form.nextFeedingAt),
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
          <span className="eyebrow">Manejo</span>
          <h1>Alimentação</h1>
          <p>Registre suplementações e acompanhe a resposta da colônia. A caixa do registro é resolvida conforme a data informada.</p>
        </div>
        <span className="count-pill">{items.length} no histórico selecionado</span>
      </section>

      <div className="content-grid">
        <section className="panel form-panel">
          <div className="panel-heading">
            <h2>Nova alimentação</h2>
            <p>Quantidade e unidade são opcionais, mas devem ser informadas juntas quando usadas.</p>
          </div>

          {colonies.length === 0 ? (
            <div className="inline-notice">Cadastre uma colônia antes de registrar alimentação.</div>
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
                <input type="datetime-local" value={form.fedAt} onChange={(e) => setForm({ ...form, fedAt: e.target.value })} />
              </label>
              <label className="field">
                <span>Tipo de alimento</span>
                <input required value={form.foodType} onChange={(e) => setForm({ ...form, foodType: e.target.value })} placeholder="Ex.: xarope 1:1" />
              </label>
              <label className="field">
                <span>Quantidade</span>
                <input min="0" step="any" type="number" value={form.quantity} onChange={(e) => setForm({ ...form, quantity: e.target.value })} placeholder="Ex.: 50" />
              </label>
              <label className="field">
                <span>Unidade</span>
                <input value={form.unit} onChange={(e) => setForm({ ...form, unit: e.target.value })} placeholder="Ex.: ml" />
              </label>
              <label className="field full">
                <span>Resposta observada</span>
                <input value={form.responseNotes} onChange={(e) => setForm({ ...form, responseNotes: e.target.value })} placeholder="Ex.: boa aceitação" />
              </label>
              <label className="field full">
                <span>Observações</span>
                <textarea rows={3} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} />
              </label>
              <label className="field full">
                <span>Próxima alimentação</span>
                <input type="datetime-local" value={form.nextFeedingAt} onChange={(e) => setForm({ ...form, nextFeedingAt: e.target.value })} />
              </label>
              <div className="form-actions full">
                <button disabled={busy || !form.colonyId || !form.foodType.trim()} type="submit">{busy ? "Salvando..." : "Registrar alimentação"}</button>
              </div>
            </form>
          )}
        </section>

        <section className="panel list-panel">
          <div className="panel-heading">
            <h2>Histórico da colônia</h2>
            <p>As alimentações mais recentes aparecem primeiro.</p>
          </div>
          {!form.colonyId ? (
            <div className="empty-list">Selecione uma colônia para consultar o histórico.</div>
          ) : loading ? (
            <div className="empty-list">Carregando alimentações...</div>
          ) : loadError ? (
            <div className="inline-notice">{loadError}</div>
          ) : items.length === 0 ? (
            <div className="empty-list">Nenhuma alimentação registrada para esta colônia.</div>
          ) : (
            <div className="record-list">
              {items.map((item) => (
                <article className="record-card" key={item.id}>
                  <div className="record-title-row">
                    <div>
                      <strong>{item.foodType}</strong>
                      <span>{formatDateTime(item.fedAt)} · {item.boxCode ? `Caixa ${item.boxCode}` : "sem caixa na data"}</span>
                    </div>
                    <span className="badge">{quantityLabel(item)}</span>
                  </div>
                  <dl>
                    <div><dt>Resposta</dt><dd>{item.responseNotes || "Não informada"}</dd></div>
                    <div><dt>Próxima</dt><dd>{item.nextFeedingAt ? formatDateTime(item.nextFeedingAt) : "Sem agendamento"}</dd></div>
                  </dl>
                  {item.notes && <p>{item.notes}</p>}
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

function quantityLabel(item: Feeding) {
  if (item.quantity == null || !item.unit) return "Sem quantidade";
  return `${item.quantity} ${item.unit}`;
}
