import { useMemo, useState, type FormEvent } from "react";
import type { CreateBoxInput, HiveBox, Meliponary } from "../types";

type BoxesPageProps = {
  items: HiveBox[];
  meliponaries: Meliponary[];
  busy: boolean;
  onCreate: (input: CreateBoxInput) => Promise<boolean>;
};

const initialForm: CreateBoxInput = {
  meliponaryId: "",
  code: "",
  model: "",
  material: "",
  locationNote: "",
  notes: "",
};

export function BoxesPage({ items, meliponaries, busy, onCreate }: BoxesPageProps) {
  const [form, setForm] = useState<CreateBoxInput>(initialForm);
  const meliponaryNames = useMemo(
    () => new Map(meliponaries.map((item) => [item.id, item.name])),
    [meliponaries],
  );

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (await onCreate(form)) {
      setForm({ ...initialForm, meliponaryId: form.meliponaryId });
    }
  }

  return (
    <div className="page-stack">
      <section className="page-heading">
        <div>
          <span className="eyebrow">Estrutura física</span>
          <h1>Caixas</h1>
          <p>Caixa é o objeto físico. A colônia mantém sua própria identidade mesmo quando troca de caixa.</p>
        </div>
        <span className="count-pill">{items.length} cadastradas</span>
      </section>

      <div className="content-grid">
        <section className="panel form-panel">
          <div className="panel-heading">
            <h2>Nova caixa</h2>
            <p>O código precisa ser único dentro do meliponário.</p>
          </div>
          {meliponaries.length === 0 ? (
            <div className="inline-notice">Cadastre um meliponário antes de adicionar caixas.</div>
          ) : (
            <form className="form-grid" onSubmit={submit}>
              <label className="field full">
                <span>Meliponário</span>
                <select required value={form.meliponaryId} onChange={(e) => setForm({ ...form, meliponaryId: e.target.value })}>
                  <option value="">Selecione...</option>
                  {meliponaries.map((item) => <option value={item.id} key={item.id}>{item.name}</option>)}
                </select>
              </label>
              <label className="field">
                <span>Código</span>
                <input required value={form.code} onChange={(e) => setForm({ ...form, code: e.target.value })} placeholder="Ex.: CX-001" />
              </label>
              <label className="field">
                <span>Modelo</span>
                <input value={form.model} onChange={(e) => setForm({ ...form, model: e.target.value })} placeholder="Ex.: INPA" />
              </label>
              <label className="field">
                <span>Material</span>
                <input value={form.material} onChange={(e) => setForm({ ...form, material: e.target.value })} placeholder="Ex.: Madeira" />
              </label>
              <label className="field">
                <span>Posição / local</span>
                <input value={form.locationNote} onChange={(e) => setForm({ ...form, locationNote: e.target.value })} placeholder="Ex.: Prateleira norte" />
              </label>
              <label className="field full">
                <span>Observações</span>
                <textarea rows={3} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} />
              </label>
              <div className="form-actions full">
                <button disabled={busy} type="submit">{busy ? "Salvando..." : "Cadastrar caixa"}</button>
              </div>
            </form>
          )}
        </section>

        <section className="panel list-panel">
          <div className="panel-heading">
            <h2>Caixas cadastradas</h2>
            <p>Ocupação atual sem perder o histórico anterior.</p>
          </div>
          {items.length === 0 ? (
            <div className="empty-list">Nenhuma caixa cadastrada ainda.</div>
          ) : (
            <div className="record-list">
              {items.map((item) => (
                <article className="record-card" key={item.id}>
                  <div className="record-title-row">
                    <div>
                      <strong>{item.code}</strong>
                      <span>{meliponaryNames.get(item.meliponaryId) || "Meliponário"}</span>
                    </div>
                    <span className={item.currentColonyCode ? "badge occupied" : "badge"}>
                      {item.currentColonyCode ? "Ocupada" : "Livre"}
                    </span>
                  </div>
                  <dl className="record-details">
                    <div><dt>Colônia atual</dt><dd>{item.currentColonyCode || "Nenhuma"}</dd></div>
                    <div><dt>Modelo</dt><dd>{item.model || "Não informado"}</dd></div>
                    <div><dt>Material</dt><dd>{item.material || "Não informado"}</dd></div>
                    <div><dt>Posição</dt><dd>{item.locationNote || "Não informada"}</dd></div>
                  </dl>
                </article>
              ))}
            </div>
          )}
        </section>
      </div>
    </div>
  );
}
