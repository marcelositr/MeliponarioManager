import { useState, type FormEvent } from "react";
import type { CreateSpeciesInput, Species } from "../types";

type SpeciesPageProps = {
  items: Species[];
  busy: boolean;
  onCreate: (input: CreateSpeciesInput) => Promise<boolean>;
};

const initialForm: CreateSpeciesInput = {
  commonName: "",
  scientificName: "",
  genus: "",
  notes: "",
};

export function SpeciesPage({ items, busy, onCreate }: SpeciesPageProps) {
  const [form, setForm] = useState<CreateSpeciesInput>(initialForm);

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (await onCreate(form)) setForm(initialForm);
  }

  return (
    <div className="page-stack">
      <section className="page-heading">
        <div>
          <span className="eyebrow">Catálogo</span>
          <h1>Espécies</h1>
          <p>Mantenha um catálogo único de espécies para que as colônias usem a mesma referência ao longo de todo o histórico.</p>
        </div>
        <span className="count-pill">{items.length} cadastradas</span>
      </section>

      <div className="content-grid">
        <section className="panel form-panel">
          <div className="panel-heading">
            <h2>Nova espécie</h2>
            <p>Use o nome popular no dia a dia e complete os dados técnicos quando souber.</p>
          </div>
          <form className="form-grid" onSubmit={submit}>
            <label className="field full">
              <span>Nome popular</span>
              <input required value={form.commonName} onChange={(e) => setForm({ ...form, commonName: e.target.value })} placeholder="Ex.: Jataí" />
            </label>
            <label className="field">
              <span>Nome científico</span>
              <input value={form.scientificName} onChange={(e) => setForm({ ...form, scientificName: e.target.value })} placeholder="Ex.: Tetragonisca angustula" />
            </label>
            <label className="field">
              <span>Gênero</span>
              <input value={form.genus} onChange={(e) => setForm({ ...form, genus: e.target.value })} placeholder="Ex.: Tetragonisca" />
            </label>
            <label className="field full">
              <span>Observações</span>
              <textarea rows={4} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} placeholder="Apelidos locais, características ou notas" />
            </label>
            <div className="form-actions full">
              <button disabled={busy} type="submit">{busy ? "Salvando..." : "Cadastrar espécie"}</button>
            </div>
          </form>
        </section>

        <section className="panel list-panel">
          <div className="panel-heading">
            <h2>Catálogo atual</h2>
            <p>Referências utilizadas pelas colônias.</p>
          </div>
          {items.length === 0 ? (
            <div className="empty-list">Nenhuma espécie cadastrada ainda.</div>
          ) : (
            <div className="record-list">
              {items.map((item) => (
                <article className="record-card" key={item.id}>
                  <div>
                    <strong>{item.commonName}</strong>
                    <span className="scientific-name">{item.scientificName || "Nome científico não informado"}</span>
                  </div>
                  <dl>
                    <div><dt>Gênero</dt><dd>{item.genus || "Não informado"}</dd></div>
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
