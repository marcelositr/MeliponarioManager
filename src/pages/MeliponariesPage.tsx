import { useState, type FormEvent } from "react";
import type { CreateMeliponaryInput, Meliponary } from "../types";

type MeliponariesPageProps = {
  items: Meliponary[];
  busy: boolean;
  onCreate: (input: CreateMeliponaryInput) => Promise<boolean>;
};

const initialForm: CreateMeliponaryInput = {
  name: "",
  responsibleName: "",
  location: "",
  notes: "",
};

export function MeliponariesPage({ items, busy, onCreate }: MeliponariesPageProps) {
  const [form, setForm] = useState<CreateMeliponaryInput>(initialForm);

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (await onCreate(form)) setForm(initialForm);
  }

  return (
    <div className="page-stack">
      <section className="page-heading">
        <div>
          <span className="eyebrow">Estrutura</span>
          <h1>Meliponários</h1>
          <p>Cadastre cada unidade de criação separadamente. Caixas e colônias permanecem vinculadas ao local correto.</p>
        </div>
        <span className="count-pill">{items.length} cadastrados</span>
      </section>

      <div className="content-grid">
        <section className="panel form-panel">
          <div className="panel-heading">
            <h2>Novo meliponário</h2>
            <p>Somente o nome é obrigatório.</p>
          </div>
          <form className="form-grid" onSubmit={submit}>
            <label className="field full">
              <span>Nome</span>
              <input required value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} placeholder="Ex.: Meliponário principal" />
            </label>
            <label className="field">
              <span>Responsável</span>
              <input value={form.responsibleName} onChange={(e) => setForm({ ...form, responsibleName: e.target.value })} placeholder="Nome do responsável" />
            </label>
            <label className="field">
              <span>Localização</span>
              <input value={form.location} onChange={(e) => setForm({ ...form, location: e.target.value })} placeholder="Sítio, bairro ou referência" />
            </label>
            <label className="field full">
              <span>Observações</span>
              <textarea rows={4} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} placeholder="Informações úteis sobre este local" />
            </label>
            <div className="form-actions full">
              <button disabled={busy} type="submit">{busy ? "Salvando..." : "Cadastrar meliponário"}</button>
            </div>
          </form>
        </section>

        <section className="panel list-panel">
          <div className="panel-heading">
            <h2>Locais cadastrados</h2>
            <p>A base física do plantel.</p>
          </div>
          {items.length === 0 ? (
            <div className="empty-list">Nenhum meliponário cadastrado ainda.</div>
          ) : (
            <div className="record-list">
              {items.map((item) => (
                <article className="record-card" key={item.id}>
                  <div>
                    <strong>{item.name}</strong>
                    <span>{item.location || "Localização não informada"}</span>
                  </div>
                  <dl>
                    <div><dt>Responsável</dt><dd>{item.responsibleName || "Não informado"}</dd></div>
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
