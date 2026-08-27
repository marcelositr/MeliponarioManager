import type { AppActions } from "../hooks/useAppData";
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
  onNavigate: (view: View) => void;
};

export function WorkspaceRouter({ activeView, data, stats, busy, actions, onNavigate }: WorkspaceRouterProps) {
  if (activeView === "dashboard") return <DashboardPage stats={stats} onNavigate={onNavigate} />;
  if (activeView === "meliponaries") return <MeliponariesPage items={data.meliponaries} busy={busy} onCreate={actions.createMeliponary} />;
  if (activeView === "species") return <SpeciesPage items={data.species} busy={busy} onCreate={actions.createSpecies} />;
  if (activeView === "colonies") return <ColoniesPage items={data.colonies} meliponaries={data.meliponaries} species={data.species} boxes={data.boxes} busy={busy} onCreate={actions.createColony} onPlace={actions.placeColony} />;
  if (activeView === "boxes") return <BoxesPage items={data.boxes} meliponaries={data.meliponaries} busy={busy} onCreate={actions.createBox} />;
  if (activeView === "inspections") return <InspectionsPage colonies={data.colonies} busy={busy} onCreate={actions.createInspection} />;
  if (activeView === "feeding") return <FeedingPage colonies={data.colonies} busy={busy} onCreate={actions.createFeeding} />;
  if (activeView === "production") return <ProductionPage colonies={data.colonies} busy={busy} onCreate={actions.createProduction} />;
  if (activeView === "history") return <EventsTimelinePage colonies={data.colonies} busy={busy} onCreate={actions.createEvent} />;
  if (activeView === "alerts") return <AlertsPage />;
  if (activeView === "genealogy") return <DivisionsPage colonies={data.colonies} busy={busy} onCreate={actions.createDivision} />;
  if (activeView === "movements") return <MovementsPage colonies={data.colonies} meliponaries={data.meliponaries} boxes={data.boxes} busy={busy} onCreateMovement={actions.createMovement} onCreateDocument={actions.createMovementDocument} />;
  if (activeView === "assets") return <AssetsPage colonies={data.colonies} boxes={data.boxes} busy={busy} onImportPhoto={actions.importInspectionPhoto} onDeletePhoto={actions.deleteInspectionPhoto} onCreateMaintenance={actions.createBoxMaintenance} />;
  if (activeView === "lifecycle") return <LifecyclePage colonies={data.colonies} busy={busy} onChange={actions.changeLifecycle} />;
  return <DataManagementPage />;
}
