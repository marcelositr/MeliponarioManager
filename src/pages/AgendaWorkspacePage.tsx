import { AgendaPage } from "./AgendaPage";
import type { Navigate, NavigationIntent } from "../lib/navigation";
import type { Colony, HiveBox, Meliponary } from "../types";

type Props = {
  meliponaries: Meliponary[];
  colonies: Colony[];
  boxes: HiveBox[];
  activeMeliponaryId: string;
  navigationIntent: NavigationIntent;
  onNavigate: Navigate;
};

export function AgendaWorkspacePage({ meliponaries, colonies, boxes, activeMeliponaryId, navigationIntent, onNavigate }: Props) {
  return <div className="page-stack">
    <div className="workspace-actions" aria-label="Atalhos da Agenda">
      <button className="button-secondary" type="button" onClick={() => onNavigate("alerts")}>Ver alertas</button>
      <button className="button-secondary" type="button" onClick={() => onNavigate("colonies")}>Abrir colônias</button>
      <button className="button-secondary" type="button" onClick={() => onNavigate("boxes")}>Abrir caixas</button>
    </div>
    <AgendaPage meliponaries={meliponaries} colonies={colonies} boxes={boxes} activeMeliponaryId={activeMeliponaryId} focusTaskId={navigationIntent.view === "agenda" ? navigationIntent.taskId ?? undefined : undefined} focusColonyId={navigationIntent.view === "agenda" ? navigationIntent.colonyId ?? undefined : undefined} />
  </div>;
}
