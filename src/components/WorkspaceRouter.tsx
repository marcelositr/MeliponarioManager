import type { AppActions, RecordStateMap } from "../hooks/useAppData";
import type { CoreData, DashboardStats, View } from "../types";
import { AlertsPage } from "../pages/AlertsPage";
import { AssetsPage } from "../pages/AssetsPage";
import { BoxesPage } from "../pages/BoxesPage";
import { ColoniesPage } from "../pages/ColoniesPage";
import { DashboardPage } from "../pages/DashboardPage";
import { DataManagementPage } from "../pages/DataManagementPage";
import { DivisionsPage } from "../pages/DivisionsPage";
import { EventsTimelinePage } from "../pages/EventsTimelinePage";
import { FeedingPage } from "../pages/FeedingPage";
import { InspectionsPage } from "../pages/InspectionsPage";
import { LifecyclePage } from "../pages/LifecyclePage";
import { MeliponariesPage } from "../pages/MeliponariesPage";
import { MovementsPage } from "../pages/MovementsPage";
import { ProductionPage } from "../pages/ProductionPage";
import { SpeciesPage } from "../pages/SpeciesPage";

type WorkspaceRouterProps = {
  activeView: View;
  data: CoreData;
  stats: DashboardStats;
  busy: boolean;
  actions: AppActions;
  recordStateMap: RecordStateMap;
  onNavigate: (view: View) => void;
};

export function WorkspaceRouter({ activeView, data, stats, busy, actions, recordStateMap, onNavigate }: WorkspaceRouterProps) {
  if (activeView === "dashboard") return <DashboardPage stats={stats} onNavigate={onNavigate} />;
  if (activeView === "meliponaries") return <MeliponariesPage items={data.meliponaries} busy={busy} onCreate={actions.createMeliponary} onEdit={actions.editMeliponary} onArchive={actions.archiveMeliponary} onReactivate={actions.reactivateMeliponary} onDelete={actions.deleteMeliponary} />;
  if (activeView === "species") return <SpeciesPage items={data.species} busy={busy} onCreate={actions.createSpecies} onEdit={actions.editSpecies} onArchive={actions.archiveSpecies} onReactivate={actions.reactivateSpecies} onDelete={actions.deleteSpecies} />;
  if (activeView === "colonies") return <ColoniesPage items={data.colonies} meliponaries={data.meliponaries} species={data.species} boxes={data.boxes} busy={busy} onCreate={actions.createColony} onPlace={actions.placeColony} onEdit={actions.editColony} onDelete={actions.deleteColony} />;
  if (activeView === "boxes") return <BoxesPage items={data.boxes} meliponaries={data.meliponaries} busy={busy} onCreate={actions.createBox} onEdit={actions.editBox} onChangeState={actions.changeBoxState} onDelete={actions.deleteBox} />;
  if (activeView === "inspections") return <InspectionsPage colonies={data.colonies} busy={busy} onCreate={actions.createInspection} recordStateMap={recordStateMap} onCorrect={actions.correctInspection} onVoid={actions.voidInspection} />;
  if (activeView === "feeding") return <FeedingPage colonies={data.colonies} busy={busy} onCreate={actions.createFeeding} recordStateMap={recordStateMap} onCorrect={actions.correctFeeding} onVoid={actions.voidFeeding} />;
  if (activeView === "production") return <ProductionPage colonies={data.colonies} busy={busy} onCreate={actions.createProduction} recordStateMap={recordStateMap} onCorrect={actions.correctProduction} onVoid={actions.voidProduction} />;
  if (activeView === "history") return <EventsTimelinePage colonies={data.colonies} busy={busy} onCreate={actions.createEvent} recordStateMap={recordStateMap} onCorrect={actions.correctEvent} onVoid={actions.voidEvent} />;
  if (activeView === "alerts") return <AlertsPage />;
  if (activeView === "genealogy") return <DivisionsPage colonies={data.colonies} busy={busy} onCreate={actions.createDivision} recordStateMap={recordStateMap} onCorrect={actions.correctDivision} onVoid={actions.voidDivision} />;
  if (activeView === "movements") return <MovementsPage colonies={data.colonies} meliponaries={data.meliponaries} boxes={data.boxes} busy={busy} recordStateMap={recordStateMap} onCreateMovement={actions.createMovement} onCreateDocument={actions.createMovementDocument} onCorrectMovement={actions.correctMovement} onVoidTransport={actions.voidTransport} onReverseMovement={actions.reverseMovement} onUpdateDocument={actions.updateMovementDocument} onVoidDocument={actions.voidMovementDocument} />;
  if (activeView === "assets") return <AssetsPage colonies={data.colonies} boxes={data.boxes} busy={busy} recordStateMap={recordStateMap} onImportPhoto={actions.importInspectionPhoto} onDeletePhoto={actions.deleteInspectionPhoto} onCreateMaintenance={actions.createBoxMaintenance} onCorrectMaintenance={actions.correctMaintenance} onVoidMaintenance={actions.voidMaintenance} />;
  if (activeView === "lifecycle") return <LifecyclePage colonies={data.colonies} busy={busy} recordStateMap={recordStateMap} onChange={actions.changeLifecycle} onReverse={actions.reverseLifecycle} />;
  return <DataManagementPage />;
}
