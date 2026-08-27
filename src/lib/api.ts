import { invoke } from "@tauri-apps/api/core";
import type { Alert, Colony, ColonyDivision, ColonyEvent, ColonyMovement, CoreData, CoreSummary, CreateBoxInput, CreateColonyEventInput, CreateColonyInput, CreateDivisionInput, CreateFeedingInput, CreateInspectionInput, CreateMeliponaryInput, CreateMovementDocumentInput, CreateMovementInput, CreateProductionInput, CreateSpeciesInput, DashboardStats, Feeding, GenealogyNode, HiveBox, Inspection, Meliponary, MovementDocument, PlaceColonyInput, ProductionRecord, Species, TimelineEntry } from "../types";
export async function loadCoreData(): Promise<CoreData> { const [meliponaries, species, colonies, boxes] = await Promise.all([invoke<Meliponary[]>("list_meliponaries"), invoke<Species[]>("list_species"), invoke<Colony[]>("list_colonies"), invoke<HiveBox[]>("list_boxes")]); return { meliponaries, species, colonies, boxes }; }
export async function loadDashboardStats(): Promise<DashboardStats> { const [summary, inspections, photos, events, divisions, feedings, production, movements, documents, maintenance, lifecycle, alerts] = await Promise.all([invoke<CoreSummary>("get_core_summary"), invoke<number>("get_inspection_count"), invoke<number>("get_inspection_photo_count"), invoke<number>("get_event_count"), invoke<number>("get_division_count"), invoke<number>("get_feeding_count"), invoke<number>("get_production_count"), invoke<number>("get_movement_count"), invoke<number>("get_movement_document_count"), invoke<number>("get_box_maintenance_count"), invoke<number>("get_lifecycle_count"), invoke<number>("get_alert_count")]); return { ...summary, inspections, photos, events, divisions, feedings, production, movements, documents, maintenance, lifecycle, alerts }; }
export const createMeliponary = (input: CreateMeliponaryInput) => invoke<Meliponary>("create_meliponary", { input });
export const createSpecies = (input: CreateSpeciesInput) => invoke<Species>("create_species", { input });
export const createBox = (input: CreateBoxInput) => invoke<HiveBox>("create_box", { input });
export const createColony = (input: CreateColonyInput) => invoke<Colony>("create_colony", { input });
export const placeColony = (input: PlaceColonyInput) => invoke("place_colony", { input });
export const createInspection = (input: CreateInspectionInput) => invoke<Inspection>("create_inspection", { input });
export const listColonyInspections = (colonyId: string) => invoke<Inspection[]>("list_colony_inspections", { colonyId });
export const createFeeding = (input: CreateFeedingInput) => invoke<Feeding>("create_feeding", { input });
export const listColonyFeedings = (colonyId: string) => invoke<Feeding[]>("list_colony_feedings", { colonyId });
export const createProductionRecord = (input: CreateProductionInput) => invoke<ProductionRecord>("create_production_record", { input });
export const listColonyProduction = (colonyId: string) => invoke<ProductionRecord[]>("list_colony_production", { colonyId });
export const createColonyEvent = (input: CreateColonyEventInput) => invoke<ColonyEvent>("create_colony_event", { input });
export const getColonyTimeline = (colonyId: string) => invoke<TimelineEntry[]>("get_colony_timeline", { colonyId });
export const listAlerts = () => invoke<Alert[]>("list_alerts");
export const createColonyDivision = (input: CreateDivisionInput) => invoke<ColonyDivision>("create_colony_division", { input });
export const listColonyDivisions = (colonyId: string) => invoke<ColonyDivision[]>("list_colony_divisions", { colonyId });
export const getColonyGenealogy = (colonyId: string) => invoke<GenealogyNode[]>("get_colony_genealogy", { colonyId });
export const createColonyMovement = (input: CreateMovementInput) => invoke<ColonyMovement>("create_colony_movement", { input });
export const listColonyMovements = (colonyId: string) => invoke<ColonyMovement[]>("list_colony_movements", { colonyId });
export const createMovementDocument = (input: CreateMovementDocumentInput) => invoke<MovementDocument>("create_movement_document", { input });
export const listMovementDocuments = (movementId: string) => invoke<MovementDocument[]>("list_movement_documents", { movementId });
