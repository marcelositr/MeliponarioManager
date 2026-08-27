import { invoke } from "@tauri-apps/api/core";
import type {
  Colony,
  ColonyEvent,
  CoreData,
  CoreSummary,
  CreateBoxInput,
  CreateColonyEventInput,
  CreateColonyInput,
  CreateFeedingInput,
  CreateInspectionInput,
  CreateMeliponaryInput,
  CreateProductionInput,
  CreateSpeciesInput,
  DashboardStats,
  Feeding,
  HiveBox,
  Inspection,
  Meliponary,
  PlaceColonyInput,
  ProductionRecord,
  Species,
  TimelineEntry,
} from "../types";

export async function loadCoreData(): Promise<CoreData> {
  const [meliponaries, species, colonies, boxes] = await Promise.all([
    invoke<Meliponary[]>("list_meliponaries"),
    invoke<Species[]>("list_species"),
    invoke<Colony[]>("list_colonies"),
    invoke<HiveBox[]>("list_boxes"),
  ]);

  return { meliponaries, species, colonies, boxes };
}

export async function loadDashboardStats(): Promise<DashboardStats> {
  const [summary, inspections, photos, events, divisions, feedings, production, movements, documents, maintenance, lifecycle, alerts] = await Promise.all([
    invoke<CoreSummary>("get_core_summary"),
    invoke<number>("get_inspection_count"),
    invoke<number>("get_inspection_photo_count"),
    invoke<number>("get_event_count"),
    invoke<number>("get_division_count"),
    invoke<number>("get_feeding_count"),
    invoke<number>("get_production_count"),
    invoke<number>("get_movement_count"),
    invoke<number>("get_movement_document_count"),
    invoke<number>("get_box_maintenance_count"),
    invoke<number>("get_lifecycle_count"),
    invoke<number>("get_alert_count"),
  ]);

  return { ...summary, inspections, photos, events, divisions, feedings, production, movements, documents, maintenance, lifecycle, alerts };
}

export function createMeliponary(input: CreateMeliponaryInput) {
  return invoke<Meliponary>("create_meliponary", { input });
}
export function createSpecies(input: CreateSpeciesInput) {
  return invoke<Species>("create_species", { input });
}
export function createBox(input: CreateBoxInput) {
  return invoke<HiveBox>("create_box", { input });
}
export function createColony(input: CreateColonyInput) {
  return invoke<Colony>("create_colony", { input });
}
export function placeColony(input: PlaceColonyInput) {
  return invoke("place_colony", { input });
}
export function createInspection(input: CreateInspectionInput) {
  return invoke<Inspection>("create_inspection", { input });
}
export function listColonyInspections(colonyId: string) {
  return invoke<Inspection[]>("list_colony_inspections", { colonyId });
}
export function createFeeding(input: CreateFeedingInput) {
  return invoke<Feeding>("create_feeding", { input });
}
export function listColonyFeedings(colonyId: string) {
  return invoke<Feeding[]>("list_colony_feedings", { colonyId });
}
export function createProductionRecord(input: CreateProductionInput) {
  return invoke<ProductionRecord>("create_production_record", { input });
}
export function listColonyProduction(colonyId: string) {
  return invoke<ProductionRecord[]>("list_colony_production", { colonyId });
}
export function createColonyEvent(input: CreateColonyEventInput) {
  return invoke<ColonyEvent>("create_colony_event", { input });
}
export function getColonyTimeline(colonyId: string) {
  return invoke<TimelineEntry[]>("get_colony_timeline", { colonyId });
}
