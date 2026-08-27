import { useCallback, useEffect, useState } from "react";
import { changeColonyLifecycle, createBox, createBoxMaintenance, createColony, createColonyDivision, createColonyEvent, createColonyMovement, createFeeding, createInspection, createMeliponary, createMovementDocument, createProductionRecord, createSpecies, deleteInspectionPhoto, importInspectionPhoto, loadCoreData, loadDashboardStats, placeColony } from "../lib/api";
import type { ChangeColonyLifecycleInput, CoreData, CreateBoxInput, CreateBoxMaintenanceInput, CreateColonyEventInput, CreateColonyInput, CreateDivisionInput, CreateFeedingInput, CreateInspectionInput, CreateMeliponaryInput, CreateMovementDocumentInput, CreateMovementInput, CreateProductionInput, CreateSpeciesInput, DashboardStats, ImportInspectionPhotoInput, PlaceColonyInput } from "../types";

const emptyData: CoreData = { meliponaries: [], species: [], colonies: [], boxes: [] };
const emptyStats: DashboardStats = { meliponaries: 0, species: 0, colonies: 0, boxes: 0, inspections: 0, photos: 0, events: 0, divisions: 0, feedings: 0, production: 0, movements: 0, documents: 0, maintenance: 0, lifecycle: 0, alerts: 0 };
export type Feedback = { kind: "success" | "error"; text: string } | null;

export function useAppData() {
  const [data, setData] = useState<CoreData>(emptyData);
  const [stats, setStats] = useState<DashboardStats>(emptyStats);
  const [connectionStatus, setConnectionStatus] = useState("Conectando...");
  const [busy, setBusy] = useState(false);
  const [feedback, setFeedback] = useState<Feedback>(null);

  const refresh = useCallback(async () => {
    const [coreData, dashboardStats] = await Promise.all([loadCoreData(), loadDashboardStats()]);
    setData(coreData);
    setStats(dashboardStats);
    setConnectionStatus("Conectado");
  }, []);

  useEffect(() => {
    refresh().catch(() => {
      setConnectionStatus("Abra pelo Tauri");
      setFeedback({ kind: "error", text: "Não foi possível acessar o banco local. Execute a aplicação pelo Tauri." });
    });
  }, [refresh]);

  async function runMutation(action: () => Promise<unknown>, successMessage: string): Promise<boolean> {
    setBusy(true);
    setFeedback(null);
    try {
      await action();
      await refresh();
      setFeedback({ kind: "success", text: successMessage });
      return true;
    } catch (error) {
      setFeedback({ kind: "error", text: readableError(error) });
      return false;
    } finally {
      setBusy(false);
    }
  }

  const actions = {
    createMeliponary: (input: CreateMeliponaryInput) => runMutation(() => createMeliponary(input), "Meliponário cadastrado com sucesso."),
    createSpecies: (input: CreateSpeciesInput) => runMutation(() => createSpecies(input), "Espécie cadastrada com sucesso."),
    createBox: (input: CreateBoxInput) => runMutation(() => createBox(input), "Caixa cadastrada com sucesso."),
    createColony: (input: CreateColonyInput) => runMutation(() => createColony(input), "Colônia cadastrada com sucesso."),
    placeColony: (input: PlaceColonyInput) => runMutation(() => placeColony(input), "Ocupação de caixa registrada e histórico preservado."),
    createInspection: (input: CreateInspectionInput) => runMutation(() => createInspection(input), "Inspeção registrada com sucesso."),
    createFeeding: (input: CreateFeedingInput) => runMutation(() => createFeeding(input), "Alimentação registrada com sucesso."),
    createProduction: (input: CreateProductionInput) => runMutation(() => createProductionRecord(input), "Produção registrada com sucesso."),
    createEvent: (input: CreateColonyEventInput) => runMutation(() => createColonyEvent(input), "Evento registrado e incluído na timeline."),
    createDivision: (input: CreateDivisionInput) => runMutation(() => createColonyDivision(input), "Divisão registrada e genealogia atualizada."),
    createMovement: (input: CreateMovementInput) => runMutation(() => createColonyMovement(input), "Movimentação registrada e rastreabilidade atualizada."),
    createMovementDocument: (input: CreateMovementDocumentInput) => runMutation(() => createMovementDocument(input), "Documento vinculado à movimentação."),
    importInspectionPhoto: (input: ImportInspectionPhotoInput) => runMutation(() => importInspectionPhoto(input), "Foto importada para o armazenamento gerenciado."),
    deleteInspectionPhoto: (photoId: string) => runMutation(() => deleteInspectionPhoto(photoId), "Foto removida com segurança."),
    createBoxMaintenance: (input: CreateBoxMaintenanceInput) => runMutation(() => createBoxMaintenance(input), "Manutenção da caixa registrada."),
    changeLifecycle: (input: ChangeColonyLifecycleInput) => runMutation(() => changeColonyLifecycle(input), "Ciclo de vida atualizado e histórico preservado."),
  };

  return { data, stats, connectionStatus, busy, feedback, setFeedback, refresh, actions };
}

export type AppActions = ReturnType<typeof useAppData>["actions"];

function readableError(error: unknown) {
  if (typeof error === "string" && error.trim()) return error;
  if (error instanceof Error && error.message) return error.message;
  return "Não foi possível concluir a operação.";
}
