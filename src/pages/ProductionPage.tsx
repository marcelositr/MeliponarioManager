import { useEffect, useState, type FormEvent } from "react";
import { listColonyProduction } from "../lib/api";
import type { Colony, CreateProductionInput, ProductionRecord } from "../types";

type ProductionPageProps = {
  colonies: Colony[];
  busy: boolean;
  onCreate: (input: CreateProductionInput) => Promise<boolean>;
};

type ProductionForm = {
  colonyId: string;
  harvestedAt: string;
  productType: string;
  quantity: string;
  unit: string;
  purpose: string;
  notes: string;
};

const initialForm: ProductionForm = {
  colonyId: "",
  harvestedAt: "",
  productType: "honey",
  quantity: "",
  unit: "ml",
  purpose: "",
  notes: "",
};

const productOptions = [
  ["honey", "Mel"],
  ["pollen", "Pólen"],
  ["propolis", "Própolis"],
  ["wax", "Cera"],
  ["cerumen", "Cerume"],
  ["other", "Outro produto"],
] as const;

export function ProductionPage({ colonies, busy, onCreate }: ProductionPageProps) {
  const [form, setForm] = useState<ProductionForm>(initialForm);
  const [items, setItems] = useState<ProductionRecord[]>([]);
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
    listColonyProduction(form.colonyId)
      .then((data) => {
        if (!cancelled) setItems(data);
      })
      .catch(() => {
        if (!cancelled) setLoadError("Não foi possível carregar a produção desta colônia.");
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
    setItems(await listColonyProduction(colonyId));
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    const input: CreateProductionInput = {
      colonyId: form.colonyId,
      harvestedAt: normalizeDateTime(form.harvestedAt),
      productType: form.productType,
      quantity: Number(form.quantity),
      unit: form.unit,
      purpose: form.purpose,
      notes: form.notes,
    };

    if (await onCreate(input)) {
      const colonyId = form.colonyId;
      const productType = form.productType;
      const unit = form.unit;
      setForm({ ...initialForm, colonyId, productType, unit });
      await reload(colonyId);
    }
  }

  return (
    <div className="page-stack">
      <section className="page-heading">
        <div>
          <span className="eyebrow">Produção</span>
          <h1>Colheitas</h1>
          <p>Registre somente produções quantificadas. A caixa exibida no histórico corresponde à ocupação da colônia na data da colheita.</p>
        </div>
        <span className="count-pill">{items.length} no histórico selecionado</span>
      </section>

      <div className="content-grid">
        <section className="panel form-panel">
          <div className="panel-heading">
            <h2>Nova produção</h2>
            <p>Produto, quantidade e unidade são obrigatórios para manter os registros agregáveis no futuro.</p>
          </div>

          {colonies.length === 0 ? (
            <div className="inline-notice">Cadastre uma colônia antes de registrar produção.</div>
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
                <input type="datetime-local" value={form.harvestedAt} onChange={(e) => setForm({ ...form, harvestedAt: e.target.value })} />
              </label>
              <label className="field">
                <span>Produto</span>
                <select value={form.productType} onChange={(e) => setForm({ ...form, productType: e.target.value })}>
                  {productOptions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}
                </select>
              </label>
              <label className="field">
                <span>Quantidade</span>
                <input required min="0.000001" step="any" type="number" value={form.quantity} onChange={(e) => setForm({ ...form, quantity: e.target.value })} placeholder="Ex.: 120" />
              </label>
              <label className="field">
                <span>Unidade</span>
                <input required value={form.unit} onChange={(e) => setForm({ ...form, unit: e.target.value })} placeholder="Ex.: ml, g, kg" />
              </label>
              <label className="field full">
                <span>Destino ou finalidade</span>
                <input value={form.purpose} onChange={(e) => setForm({ ...form, purpose: e.target.value })} placeholder="Ex.: consumo, amostra, venda" />
              </label>
              <label className="field full">
                <span>Observações</span>
                <textarea rows={3} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} />
              </label>
              <div className="form-actions full">
                <button disabled={busy || !form.colonyId || !form.quantity || !form.unit.trim()} type="submit">{busy ? "Salvando..." : "Registrar produção"}</button>
              </div>
            </form>
          )}
        </section>

        <section className="panel list-panel">
          <div className="panel-heading">
            <h2>Histórico da colônia</h2>
            <p>As colheitas mais recentes aparecem primeiro.</p>
          </div>
          {!form.colonyId ? (
            <div className="empty-list">Selecione uma colônia para consultar o histórico.</div>
          ) : loading ? (
            <div className="empty-list">Carregando produção...</div>
          ) : loadError ? (
            <div className="inline-notice">{loadError}</div>
          ) : items.length === 0 ? (
            <div className="empty-list">Nenhuma produção registrada para esta colônia.</div>
          ) : (
            <div className="record-list">
              {items.map((item) => (
                <article className="record-card" key={item.id}>
                  <div className="record-title-row">
                    <div>
                      <strong>{productLabel(item.productType)}</strong>
                      <span>{formatDateTime(item.harvestedAt)} · {item.boxCode ? `Caixa ${item.boxCode}` : "sem caixa na data"}</span>
                    </div>
                    <span className="badge">{item.quantity} {item.unit}</span>
                  </div>
                  <dl>
                    <div><dt>Finalidade</dt><dd>{item.purpose || "Não informada"}</dd></div>
                    <div><dt>Colônia</dt><dd>{item.colonyCode}</dd></div>
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

function productLabel(value: string) {
  return productOptions.find(([key]) => key === value)?.[1] || value;
}
