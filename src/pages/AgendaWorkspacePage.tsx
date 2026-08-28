import { AgendaPage } from "./AgendaPage";
import type { Colony, HiveBox, Meliponary, View } from "../types";

type Props = {
  meliponaries: Meliponary[];
  colonies: Colony[];
  boxes: HiveBox[];
  activeMeliponaryId: string;
  onNavigate: (view: View) => void;
};

export function AgendaWorkspacePage({ meliponaries, colonies, boxes, activeMeliponaryId, onNavigate }: Props) {
  return <div className="page-stack">
    <div className="workspace-actions" aria-label="Atalhos da Agenda">
      <button className="button-secondary" type="button" onClick={() => onNavigate("alerts")}>Ver alertas</button>
      <button className="button-secondary" type="button" onClick={() => onNavigate("colonies")}>Abrir colônias</button>
      <button className="button-secondary" type="button" onClick={() => onNavigate("boxes")}>Abrir caixas</button>
    </div>
    <AgendaPage meliponaries={meliponaries} colonies={colonies} boxes={boxes} activeMeliponaryId={activeMeliponaryId} />
  </div>;
}
