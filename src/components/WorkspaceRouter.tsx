import type { AppActions, RecordStateMap } from "../hooks/useAppData";
import type { Navigate, NavigationIntent } from "../lib/navigation";
import type { CoreData, DashboardStats, View } from "../types";
import { AgendaWorkspacePage } from "../pages/AgendaWorkspacePage";
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
  navigationIntent: NavigationIntent;
  activeMeliponaryId: string;
  data: CoreData;
  stats: DashboardStats;
  busy: boolean;
  actions: AppActions;
  recordStateMap: RecordStateMap;
  onNavigate: Navigate;
};

export function WorkspaceRouter({ activeView, navigationIntent, activeMeliponaryId, data, stats, busy, actions, recordStateMap, onNavigate }: WorkspaceRouterProps) {
  const scopedMeliponaries = activeMeliponaryId ? data.meliponaries.filter((item) => item.id === activeMeliponaryId) : data.meliponaries;
  const scopedColonies = activeMeliponaryId ? data.colonies.filter((item) => item.meliponaryId === activeMeliponaryId) : data.colonies;
  const scopedBoxes = activeMeliponaryId ? data.boxes.filter((item) => item.meliponaryId === activeMeliponaryId) : data.boxes;
  const contextualColonies = navigationIntent.view === activeView && navigationIntent.colonyId ? scopedColonies.filter((item) => item.id === navigationIntent.colonyId) : scopedColonies;
  const contextualBoxes = navigationIntent.view === activeView && navigationIntent.boxId ? scopedBoxes.filter((item) => item.id === navigationIntent.boxId) : scopedBoxes;
  const contextualMeliponaries = navigationIntent.view === activeView && navigationIntent.meliponaryId ? data.meliponaries.filter((item) => item.id === navigationIntent.meliponaryId) : scopedMeliponaries;
  const autoCreate = navigationIntent.view === activeView && navigationIntent.action === "create";

  if (activeView === "dashboard") return <DashboardPage stats={stats} data={data} activeMeliponaryId={activeMeliponaryId} onNavigate={onNavigate} />;
  if (activeView === "agenda") return <AgendaWorkspacePage meliponaries={scopedMeliponaries} colonies={scopedColonies} boxes={scopedBoxes} activeMeliponaryId={activeMeliponaryId} navigationIntent={navigationIntent} onNavigate={onNavigate} />;
  if (activeView === "meliponaries") return <MeliponariesPage items={contextualMeliponaries} busy={busy} onCreate={actions.createMeliponary} onEdit={actions.editMeliponary} onArchive={actions.archiveMeliponary} onReactivate={actions.reactivateMeliponary} onDelete={actions.deleteMeliponary} onNavigate={onNavigate} />;
  if (activeView === "species") return <SpeciesPage items={data.species} busy={busy} onCreate={actions.createSpecies} onEdit={actions.editSpecies} onArchive={actions.archiveSpecies} onReactivate={actions.reactivateSpecies} onDelete={actions.deleteSpecies} />;
  if (activeView === "colonies") return <ColoniesPage items={contextualColonies} meliponaries={scopedMeliponaries} species={data.species} boxes={scopedBoxes} busy={busy} onCreate={actions.createColony} onPlace={actions.placeColony} onEdit={actions.editColony} onDelete={actions.deleteColony} onNavigate={onNavigate} />;
  if (activeView === "boxes") return <BoxesPage items={contextualBoxes} meliponaries={scopedMeliponaries} busy={busy} onCreate={actions.createBox} onEdit={actions.editBox} onChangeState={actions.changeBoxState} onDelete={actions.deleteBox} onNavigate={onNavigate} />;
  if (activeView === "inspections") return <InspectionsPage colonies={contextualColonies} busy={busy} autoCreate={autoCreate} onCreate={actions.createInspection} recordStateMap={recordStateMap} onCorrect={actions.correctInspection} onVoid={actions.voidInspection} />;
  if (activeView === "feeding") return <FeedingPage colonies={contextualColonies} busy={busy} autoCreate={autoCreate} onCreate={actions.createFeeding} recordStateMap={recordStateMap} onCorrect={actions.correctFeeding} onVoid={actions.voidFeeding} />;
  if (activeView === "production") return <ProductionPage colonies={contextualColonies} busy={busy} onCreate={actions.createProduction} recordStateMap={recordStateMap} onCorrect={actions.correctProduction} onVoid={actions.voidProduction} />;
  if (activeView === "history") return <EventsTimelinePage colonies={contextualColonies} busy={busy} onCreate={actions.createEvent} recordStateMap={recordStateMap} onCorrect={actions.correctEvent} onVoid={actions.voidEvent} />;
  if (activeView === "alerts") return <AlertsPage activeMeliponaryId={activeMeliponaryId} onNavigate={onNavigate} />;
  if (activeView === "genealogy") return <DivisionsPage colonies={contextualColonies} busy={busy} onCreate={actions.createDivision} recordStateMap={recordStateMap} onCorrect={actions.correctDivision} onVoid={actions.voidDivision} />;
  if (activeView === "movements") return <MovementsPage colonies={contextualColonies} meliponaries={data.meliponaries} boxes={data.boxes} busy={busy} recordStateMap={recordStateMap} onCreateMovement={actions.createMovement} onCreateDocument={actions.createMovementDocument} onCorrectMovement={actions.correctMovementDetails} onVoidTransport={actions.voidTransport} onReverseMovement={actions.reverseMovement} onUpdateDocument={actions.updateMovementDocument} onVoidDocument={actions.voidMovementDocument} />;
  if (activeView === "assets") return <AssetsPage colonies={contextualColonies} boxes={contextualBoxes} busy={busy} recordStateMap={recordStateMap} onImportPhoto={actions.importInspectionPhoto} onDeletePhoto={actions.deleteInspectionPhoto} onCreateMaintenance={actions.createBoxMaintenance} onCorrectMaintenance={actions.correctMaintenance} onVoidMaintenance={actions.voidMaintenance} />;
  if (activeView === "lifecycle") return <LifecyclePage colonies={contextualColonies} busy={busy} recordStateMap={recordStateMap} onChange={actions.changeLifecycle} onReverse={actions.reverseLifecycle} />;
  return <DataManagementPage />;
}
