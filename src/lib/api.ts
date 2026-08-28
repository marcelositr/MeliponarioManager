import { invoke } from "@tauri-apps/api/core";
import type {
  Alert, AuditRecord, BoxMaintenance, BoxStateRecord, ChangeBoxStateInput, ChangeColonyLifecycleInput,
  Colony, ColonyDivision, ColonyEvent, ColonyLifecycleRecord, ColonyMovement, CoreData, CoreSummary,
  CorrectDivisionInput, CorrectEventInput, CorrectFeedingInput, CorrectInspectionInput,
  CorrectMaintenanceInput, CorrectMovementDetailsInput, CorrectOccupancyInput, CorrectProductionInput,
  CreateBoxInput, CreateBoxMaintenanceInput, CreateColonyEventInput, CreateColonyInput,
  CreateDivisionInput, CreateFeedingInput, CreateInspectionInput, CreateMeliponaryInput,
  CreateMovementDocumentInput, CreateMovementInput, CreateProductionInput, CreateSpeciesInput,
  DashboardStats, EditBoxInput, EditColonyInput, EditMeliponaryInput, EditSpeciesInput, EntityActionInput,
  Feeding, GenealogyNode, HiveBox, ImportInspectionPhotoInput, Inspection, InspectionPhoto, Meliponary,
  MovementDocument, PlaceColonyInput, ProductionRecord, RecordAdminState, ReverseRecordInput, Species,
  TimelineEntry, UpdateMovementDocumentInput, VoidDivisionInput, VoidRecordInput,
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
    invoke<CoreSummary>("get_core_summary"), invoke<number>("get_inspection_count"),
    invoke<number>("get_inspection_photo_count"), invoke<number>("get_event_count"),
    invoke<number>("get_division_count"), invoke<number>("get_feeding_count"),
    invoke<number>("get_production_count"), invoke<number>("get_movement_count"),
    invoke<number>("get_movement_document_count"), invoke<number>("get_box_maintenance_count"),
    invoke<number>("get_lifecycle_count"), invoke<number>("get_alert_count"),
  ]);
  return { ...summary, inspections, photos, events, divisions, feedings, production, movements, documents, maintenance, lifecycle, alerts };
}

export const createMeliponary = (input: CreateMeliponaryInput) => invoke<Meliponary>("create_meliponary", { input });
export const createSpecies = (input: CreateSpeciesInput) => invoke<Species>("create_species", { input });
export const createBox = (input: CreateBoxInput) => invoke<HiveBox>("create_box", { input });
export const changeBoxState = (input: ChangeBoxStateInput) => invoke<BoxStateRecord>("change_box_state", { input });
export const listBoxStateHistory = (boxId: string) => invoke<BoxStateRecord[]>("list_box_state_history", { boxId });
export const createColony = (input: CreateColonyInput) => invoke<Colony>("create_colony", { input });
export const placeColony = (input: PlaceColonyInput) => invoke("place_colony", { input });
export const createInspection = (input: CreateInspectionInput) => invoke<Inspection>("create_inspection", { input });
export const listColonyInspections = (colonyId: string) => invoke<Inspection[]>("list_colony_inspections", { colonyId });
export const importInspectionPhoto = (input: ImportInspectionPhotoInput) => invoke<InspectionPhoto>("import_inspection_photo", { input });
export const listInspectionPhotos = (inspectionId: string) => invoke<InspectionPhoto[]>("list_inspection_photos", { inspectionId });
export const listColonyPhotos = (colonyId: string) => invoke<InspectionPhoto[]>("list_colony_photos", { colonyId });
export const deleteInspectionPhoto = (photoId: string) => invoke<void>("delete_inspection_photo", { photoId });
export const createFeeding = (input: CreateFeedingInput) => invoke<Feeding>("create_feeding", { input });
export const listColonyFeedings = (colonyId: string) => invoke<Feeding[]>("list_colony_feedings", { colonyId });
export const createProductionRecord = (input: CreateProductionInput) => invoke<ProductionRecord>("create_production_record", { input });
export const listColonyProduction = (colonyId: string) => invoke<ProductionRecord[]>("list_colony_production", { colonyId });
export const createColonyEvent = (input: CreateColonyEventInput) => invoke<ColonyEvent>("create_colony_event", { input });
export const listColonyEvents = (colonyId: string) => invoke<ColonyEvent[]>("list_colony_events", { colonyId });
export const getColonyTimeline = (colonyId: string) => invoke<TimelineEntry[]>("get_colony_timeline", { colonyId });
export const listAlerts = () => invoke<Alert[]>("list_alerts");
export const createColonyDivision = (input: CreateDivisionInput) => invoke<ColonyDivision>("create_colony_division", { input });
export const listColonyDivisions = (colonyId: string) => invoke<ColonyDivision[]>("list_colony_divisions", { colonyId });
export const getColonyGenealogy = (colonyId: string) => invoke<GenealogyNode[]>("get_colony_genealogy", { colonyId });
export const createColonyMovement = (input: CreateMovementInput) => invoke<ColonyMovement>("create_colony_movement", { input });
export const listColonyMovements = (colonyId: string) => invoke<ColonyMovement[]>("list_colony_movements", { colonyId });
export const createMovementDocument = (input: CreateMovementDocumentInput) => invoke<MovementDocument>("create_movement_document", { input });
export const listMovementDocuments = (movementId: string) => invoke<MovementDocument[]>("list_movement_documents", { movementId });
export const createBoxMaintenance = (input: CreateBoxMaintenanceInput) => invoke<BoxMaintenance>("create_box_maintenance", { input });
export const listBoxMaintenance = (boxId: string) => invoke<BoxMaintenance[]>("list_box_maintenance", { boxId });
export const changeColonyLifecycle = (input: ChangeColonyLifecycleInput) => invoke<ColonyLifecycleRecord>("change_colony_lifecycle", { input });
export const listColonyLifecycle = (colonyId: string) => invoke<ColonyLifecycleRecord[]>("list_colony_lifecycle", { colonyId });

export const editMeliponary = (input: EditMeliponaryInput) => invoke<Meliponary>("edit_meliponary", { input });
export const archiveMeliponary = (input: EntityActionInput) => invoke<Meliponary>("archive_meliponary", { input });
export const reactivateMeliponary = (input: EntityActionInput) => invoke<Meliponary>("reactivate_meliponary", { input });
export const deleteMeliponary = (input: EntityActionInput) => invoke<void>("delete_meliponary", { input });
export const editSpecies = (input: EditSpeciesInput) => invoke<Species>("edit_species", { input });
export const archiveSpecies = (input: EntityActionInput) => invoke<Species>("archive_species", { input });
export const reactivateSpecies = (input: EntityActionInput) => invoke<Species>("reactivate_species", { input });
export const deleteSpecies = (input: EntityActionInput) => invoke<void>("delete_species", { input });
export const editBox = (input: EditBoxInput) => invoke<HiveBox>("edit_box", { input });
export const deleteBox = (input: EntityActionInput) => invoke<void>("delete_box", { input });
export const editColony = (input: EditColonyInput) => invoke<Colony>("edit_colony", { input });
export const deleteColony = (input: EntityActionInput) => invoke<void>("delete_colony", { input });
export const listAuditRecords = (entityType: string, entityId: string) => invoke<AuditRecord[]>("list_audit_records", { entityType, entityId });
export const listRecordAdminStates = () => invoke<RecordAdminState[]>("list_record_admin_states");

export const correctInspection = (input: CorrectInspectionInput) => invoke<void>("correct_inspection", { input });
export const voidInspection = (input: VoidRecordInput) => invoke<void>("void_inspection", { input });
export const correctFeeding = (input: CorrectFeedingInput) => invoke<void>("correct_feeding", { input });
export const voidFeeding = (input: VoidRecordInput) => invoke<void>("void_feeding", { input });
export const correctProductionRecord = (input: CorrectProductionInput) => invoke<void>("correct_production_record", { input });
export const voidProductionRecord = (input: VoidRecordInput) => invoke<void>("void_production_record", { input });
export const correctBoxMaintenance = (input: CorrectMaintenanceInput) => invoke<void>("correct_box_maintenance", { input });
export const voidBoxMaintenance = (input: VoidRecordInput) => invoke<void>("void_box_maintenance", { input });
export const correctColonyEvent = (input: CorrectEventInput) => invoke<void>("correct_colony_event", { input });
export const voidColonyEvent = (input: VoidRecordInput) => invoke<void>("void_colony_event", { input });
export const correctMovementDetails = (input: CorrectMovementDetailsInput) => invoke<void>("correct_movement_details", { input });
export const voidTransport = (input: VoidRecordInput) => invoke<void>("void_transport", { input });
export const updateMovementDocument = (input: UpdateMovementDocumentInput) => invoke<void>("update_movement_document", { input });
export const voidMovementDocument = (input: VoidRecordInput) => invoke<void>("void_movement_document", { input });
export const correctColonyDivision = (input: CorrectDivisionInput) => invoke<void>("correct_colony_division", { input });
export const voidColonyDivision = (input: VoidDivisionInput) => invoke<void>("void_colony_division", { input });
export const correctBoxOccupancy = (input: CorrectOccupancyInput) => invoke<void>("correct_box_occupancy", { input });
export const reverseColonyLifecycle = (input: ReverseRecordInput) => invoke<void>("reverse_colony_lifecycle", { input });
export const reverseColonyMovement = (input: ReverseRecordInput) => invoke<void>("reverse_colony_movement", { input });
