import { useMemo, useState, type FormEvent } from "react";
import type {
  Colony,
  CreateColonyInput,
  HiveBox,
  Meliponary,
  PlaceColonyInput,
  Species,
} from "../types";

type ColoniesPageProps = {
  items: Colony[];
  meliponaries: Meliponary[];
  species: Species[];
  boxes: HiveBox[];
  busy: boolean;
  onCreate: (input: CreateColonyInput) => Promise<boolean>;
  onPlace: (input: PlaceColonyInput) => Promise<boolean>;
};

const originOptions = [
  ["acquisition", "Aquisição"],
  ["multiplication", "Multiplicação"],
  ["transfer", "Transferência"],
  ["rescue", "Resgate"],
  ["authorized_capture", "Captura autorizada"],
  ["historical", "Registro histórico"],
  ["other", "Outra origem"],
] as const;

const initialForm: CreateColonyInput = {
  meliponaryId: "",
  speciesId: "",
  code: "",
  originType: "acquisition",
  originNotes: "",
  installedAt: "",
  motherColonyId: "",
  notes: "",
};

const initialPlacement: PlaceColonyInput = {
  colonyId: "",
  boxId: "",
  startedAt: "",
  reason: "",
  notes: "",
};

export function ColoniesPage({ items, meliponaries, species, boxes, busy, onCreate, onPlace }: ColoniesPageProps) {
  const [form, setForm] = useState<CreateColonyInput>(initialForm);
  const [placement, setPlacement] = useState<PlaceColonyInput>(initialPlacement);

  const meliponaryNames = useMemo(() => new Map(meliponaries.map((item) => [item.id, item.name])), [meliponaries]);
  const speciesNames = useMemo(() => new Map(species.map((item) => [item.id, item.commonName])), [species]);
  const selectedColony = items.find((item) => item.id === placement.colonyId);
  const availableBoxes = boxes.filter(
    (box) => !box.currentColonyCode && (!selectedColony || box.meliponaryId === selectedColony.meliponaryId),
  );
  const motherOptions = items.filter(
    (item) =>
      (!form.meliponaryId || item.meliponaryId === form.meliponaryId) &&
      (!form.speciesId || item.speciesId === form.speciesId),
  );

  async function submitColony(event: FormEvent) {
    event.preventDefault();
    if (await onCreate(form)) {
      setForm({ ...initialForm, meliponaryId: form.meliponaryId, speciesId: form.speciesId });
    }
  }

  async function submitPlacement(event: FormEvent) {
    event.preventDefault();
    if (await onPlace(placement)) setPlacement(initialPlacement);
  }

  return (
    <div className="page-stack">
      <section className="page-heading">
        <div>
          <span className="eyebrow">Plantel</span>
          <h1>Colônias</h1>
          <p>A identidade da colônia acompanha toda a sua história, mesmo quando ela muda de caixa ou de meliponário.</p>
        </div>
        <span className="count-pill">{items.length} cadastradas</span>
      </section>

      <div className="content-grid colonies-grid">
        <section className="panel form-panel">
          <div className="panel-heading">
            <h2>Nova colônia</h2>
            <p>Cadastre a entidade biológica primeiro. A caixa pode ser vinculada logo ao lado.</p>
          </div>
          {meliponaries.length === 0 || species.length === 0 ? (
            <div className="inline-notice">Para criar uma colônia, cadastre pelo menos um meliponário e uma espécie.</div>
          ) : (
            <form className="form-grid" onSubmit={submitColony}>
              <label className="field">
                <span>Meliponário</span>
                <select required value={form.meliponaryId} onChange={(e) => setForm({ ...form, meliponaryId: e.target.value, motherColonyId: "" })}>
                  <option value="">Selecione...</option>
                  {meliponaries.map((item) => <option value={item.id} key={item.id}>{item.name}</option>)}
                </select>
              </label>
              <label className="field">
                <span>Espécie</span>
                <select required value={form.speciesId} onChange={(e) => setForm({ ...form, speciesId: e.target.value, motherColonyId: "" })}>
                  <option value="">Selecione...</option>
                  {species.map((item) => <option value={item.id} key={item.id}>{item.commonName}</option>)}
                </select>
              </label>
              <label className="field">
                <span>Código</span>
                <input required value={form.code} onChange={(e) => setForm({ ...form, code: e.target.value })} placeholder="Ex.: JAT-001" />
              </label>
              <label className="field">
                <span>Origem</span>
                <select value={form.originType} onChange={(e) => setForm({ ...form, originType: e.target.value })}>
                  {originOptions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}
                </select>
              </label>
              <label className="field">
                <span>Data de instalação</span>
                <input type="date" value={form.installedAt} onChange={(e) => setForm({ ...form, installedAt: e.target.value })} />
              </label>
              <label className="field">
                <span>Colônia-mãe</span>
                <select value={form.motherColonyId} onChange={(e) => setForm({ ...form, motherColonyId: e.target.value })}>
                  <option value="">Sem vínculo</option>
                  {motherOptions.map((item) => <option value={item.id} key={item.id}>{item.code}</option>)}
                </select>
              </label>
              <label className="field full">
                <span>Detalhes da origem</span>
                <input value={form.originNotes} onChange={(e) => setForm({ ...form, originNotes: e.target.value })} placeholder="Fornecedor, divisão, resgate ou referência" />
              </label>
              <label className="field full">
                <span>Observações</span>
                <textarea rows={3} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} />
              </label>
              <div className="form-actions full">
                <button disabled={busy} type="submit">{busy ? "Salvando..." : "Cadastrar colônia"}</button>
              </div>
            </form>
          )}
        </section>

        <section className="panel placement-panel">
          <div className="panel-heading">
            <h2>Colocar ou mover para uma caixa</h2>
            <p>O sistema encerra a ocupação anterior e preserva o histórico automaticamente.</p>
          </div>
          {items.length === 0 || boxes.length === 0 ? (
            <div className="inline-notice">Cadastre uma colônia e uma caixa para registrar a ocupação.</div>
          ) : (
            <form className="form-grid" onSubmit={submitPlacement}>
              <label className="field full">
                <span>Colônia</span>
                <select required value={placement.colonyId} onChange={(e) => setPlacement({ ...placement, colonyId: e.target.value, boxId: "" })}>
                  <option value="">Selecione...</option>
                  {items.filter((item) => item.status === "active").map((item) => (
                    <option value={item.id} key={item.id}>{item.code} {item.currentBoxCode ? `· atual ${item.currentBoxCode}` : "· sem caixa"}</option>
                  ))}
                </select>
              </label>
              <label className="field full">
                <span>Caixa de destino</span>
                <select required value={placement.boxId} onChange={(e) => setPlacement({ ...placement, boxId: e.target.value })}>
                  <option value="">Selecione uma caixa livre...</option>
                  {availableBoxes.map((box) => <option value={box.id} key={box.id}>{box.code}</option>)}
                </select>
              </label>
              <label className="field">
                <span>Data</span>
                <input type="date" value={placement.startedAt} onChange={(e) => setPlacement({ ...placement, startedAt: e.target.value })} />
              </label>
              <label className="field">
                <span>Motivo</span>
                <input value={placement.reason} onChange={(e) => setPlacement({ ...placement, reason: e.target.value })} placeholder="Ex.: Instalação inicial" />
              </label>
              <label className="field full">
                <span>Observações</span>
                <textarea rows={3} value={placement.notes} onChange={(e) => setPlacement({ ...placement, notes: e.target.value })} />
              </label>
              <div className="form-actions full">
                <button disabled={busy || availableBoxes.length === 0} type="submit">{busy ? "Salvando..." : "Registrar ocupação"}</button>
              </div>
            </form>
          )}
        </section>
      </div>

      <section className="panel list-panel wide-list">
        <div className="panel-heading">
          <h2>Plantel cadastrado</h2>
          <p>Estado atual com identidade e localização separadas.</p>
        </div>
        {items.length === 0 ? (
          <div className="empty-list">Nenhuma colônia cadastrada ainda.</div>
        ) : (
          <div className="table-wrap">
            <table className="data-table">
              <thead>
                <tr><th>Código</th><th>Espécie</th><th>Meliponário</th><th>Caixa atual</th><th>Situação</th></tr>
              </thead>
              <tbody>
                {items.map((item) => (
                  <tr key={item.id}>
                    <td><strong>{item.code}</strong></td>
                    <td>{speciesNames.get(item.speciesId) || "Espécie"}</td>
                    <td>{meliponaryNames.get(item.meliponaryId) || "Meliponário"}</td>
                    <td>{item.currentBoxCode || "Sem caixa"}</td>
                    <td><span className={`badge status-${item.status}`}>{translateStatus(item.status)}</span></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </div>
  );
}

function translateStatus(status: string) {
  const labels: Record<string, string> = {
    active: "Ativa",
    weak: "Fraca",
    lost: "Perdida",
    inactive: "Inativa",
    transferred: "Transferida",
  };
  return labels[status] || status;
}
