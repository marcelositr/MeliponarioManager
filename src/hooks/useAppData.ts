import { useCallback, useEffect, useMemo, useState } from "react";
import {
  archiveMeliponary, archiveSpecies, changeBoxState, changeColonyLifecycle, correctBoxMaintenance,
  correctBoxOccupancy, correctColonyDivision, correctColonyEvent, correctFeeding, correctInspection,
  correctMovementDetails, correctProductionRecord, createBox, createBoxMaintenance, createColony,
  createColonyDivision, createColonyEvent, createColonyMovement, createFeeding, createInspection,
  createMeliponary, createMovementDocument, createProductionRecord, createSpecies, deleteBox,
  deleteColony, deleteInspectionPhoto, deleteMeliponary, deleteSpecies, editBox, editColony,
  editMeliponary, editSpecies, importInspectionPhoto, listRecordAdminStates, loadCoreData,
  loadDashboardStats, placeColony, reactivateMeliponary, reactivateSpecies, reverseColonyLifecycle,
  reverseColonyMovement, updateMovementDocument, voidBoxMaintenance, voidColonyDivision,
  voidColonyEvent, voidFeeding, voidInspection, voidMovementDocument, voidProductionRecord,
  voidTransport,
} from "../lib/api";
import { publicError } from "../lib/presentation";
import { importSpeciesCsv as importSpeciesCsvFile, type SpeciesImportResult } from "../lib/species-import";
import type {
  ChangeBoxStateInput, ChangeColonyLifecycleInput, CoreData, CorrectDivisionInput, CorrectEventInput,
  CorrectFeedingInput, CorrectInspectionInput, CorrectMaintenanceInput, CorrectMovementDetailsInput,
  CorrectOccupancyInput, CorrectProductionInput, CreateBoxInput, CreateBoxMaintenanceInput,
  CreateColonyEventInput, CreateColonyInput, CreateDivisionInput, CreateFeedingInput,
  CreateInspectionInput, CreateMeliponaryInput, CreateMovementDocumentInput, CreateMovementInput,
  CreateProductionInput, CreateSpeciesInput, DashboardStats, EditBoxInput, EditColonyInput,
  EditMeliponaryInput, EditSpeciesInput, EntityActionInput, ImportInspectionPhotoInput,
  PlaceColonyInput, RecordAdminState, ReverseRecordInput, UpdateMovementDocumentInput,
  VoidDivisionInput, VoidRecordInput,
} from "../types";

const emptyData: CoreData = { meliponaries: [], species: [], colonies: [], boxes: [] };
const emptyStats: DashboardStats = { meliponaries: 0, species: 0, colonies: 0, boxes: 0, inspections: 0, photos: 0, events: 0, divisions: 0, feedings: 0, production: 0, movements: 0, documents: 0, maintenance: 0, lifecycle: 0, alerts: 0 };
export type Feedback = { kind: "success" | "error"; text: string } | null;

export function useAppData() {
  const [data, setData] = useState<CoreData>(emptyData);
  const [stats, setStats] = useState<DashboardStats>(emptyStats);
  const [recordStates, setRecordStates] = useState<RecordAdminState[]>([]);
  const [connectionStatus, setConnectionStatus] = useState("Conectando...");
  const [busy, setBusy] = useState(false);
  const [feedback, setFeedback] = useState<Feedback>(null);

  const refresh = useCallback(async () => {
    const [coreData, dashboardStats, adminStates] = await Promise.all([loadCoreData(), loadDashboardStats(), listRecordAdminStates()]);
    setData(coreData); setStats(dashboardStats); setRecordStates(adminStates); setConnectionStatus("Conectado");
  }, []);

  useEffect(() => {
    refresh().catch(() => {
      setConnectionStatus("Abra pelo Tauri");
      setFeedback({ kind: "error", text: "Não foi possível acessar o banco local. Execute a aplicação pelo Tauri." });
    });
  }, [refresh]);

  const recordStateMap = useMemo(() => new Map(recordStates.map((item) => [`${item.entityType}:${item.entityId}`, item])), [recordStates]);

  async function runMutation(action: () => Promise<unknown>, successMessage: string): Promise<boolean> {
    setBusy(true); setFeedback(null);
    try { await action(); await refresh(); setFeedback({ kind: "success", text: successMessage }); return true; }
    catch (error) { setFeedback({ kind: "error", text: publicError(error, "Não foi possível concluir a operação. Verifique os dados e tente novamente.") }); return false; }
    finally { setBusy(false); }
  }

  const actions = {
    createMeliponary: (input: CreateMeliponaryInput) => runMutation(() => createMeliponary(input), "Meliponário cadastrado com sucesso."),
    createSpecies: (input: CreateSpeciesInput) => runMutation(() => createSpecies(input), "Espécie cadastrada com sucesso."),
    importSpeciesCsv: async (sourcePath: string): Promise<SpeciesImportResult | null> => {
      setBusy(true); setFeedback(null);
      try {
        const result = await importSpeciesCsvFile(sourcePath);
        await refresh();
        const importedLabel = result.importedRows === 1 ? "1 espécie importada." : `${result.importedRows} espécies importadas.`;
        const duplicateLabel = result.duplicateRows > 0
          ? ` ${result.duplicateRows} duplicada${result.duplicateRows === 1 ? "" : "s"} ignorada${result.duplicateRows === 1 ? "" : "s"}.`
          : "";
        setFeedback({ kind: "success", text: `${importedLabel}${duplicateLabel}` });
        return result;
      } catch (error) {
        setFeedback({ kind: "error", text: publicError(error, "Não foi possível importar a lista de espécies.") });
        return null;
      } finally {
        setBusy(false);
      }
    },
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
    changeBoxState: (input: ChangeBoxStateInput) => runMutation(() => changeBoxState(input), "Estado físico da caixa atualizado."),
    editMeliponary: (input: EditMeliponaryInput) => runMutation(() => editMeliponary(input), "Meliponário atualizado e alteração auditada."),
    archiveMeliponary: (input: EntityActionInput) => runMutation(() => archiveMeliponary(input), "Meliponário arquivado sem apagar o histórico."),
    reactivateMeliponary: (input: EntityActionInput) => runMutation(() => reactivateMeliponary(input), "Meliponário reativado."),
    deleteMeliponary: (input: EntityActionInput) => runMutation(() => deleteMeliponary(input), "Cadastro vazio de meliponário excluído."),
    editSpecies: (input: EditSpeciesInput) => runMutation(() => editSpecies(input), "Espécie atualizada e alteração auditada."),
    archiveSpecies: (input: EntityActionInput) => runMutation(() => archiveSpecies(input), "Espécie arquivada sem perder referências históricas."),
    reactivateSpecies: (input: EntityActionInput) => runMutation(() => reactivateSpecies(input), "Espécie reativada."),
    deleteSpecies: (input: EntityActionInput) => runMutation(() => deleteSpecies(input), "Cadastro vazio de espécie excluído."),
    editBox: (input: EditBoxInput) => runMutation(() => editBox(input), "Dados da caixa atualizados."),
    deleteBox: (input: EntityActionInput) => runMutation(() => deleteBox(input), "Caixa nunca utilizada excluída."),
    editColony: (input: EditColonyInput) => runMutation(() => editColony(input), "Dados descritivos da colônia atualizados."),
    deleteColony: (input: EntityActionInput) => runMutation(() => deleteColony(input), "Colônia sem histórico excluída."),
    correctInspection: (input: CorrectInspectionInput) => runMutation(() => correctInspection(input), "Inspeção corrigida; o antes/depois foi auditado."),
    voidInspection: (input: VoidRecordInput) => runMutation(() => voidInspection(input), "Inspeção anulada e preservada no histórico."),
    correctFeeding: (input: CorrectFeedingInput) => runMutation(() => correctFeeding(input), "Alimentação corrigida e auditada."),
    voidFeeding: (input: VoidRecordInput) => runMutation(() => voidFeeding(input), "Alimentação anulada e preservada no histórico."),
    correctProduction: (input: CorrectProductionInput) => runMutation(() => correctProductionRecord(input), "Produção corrigida e auditada."),
    voidProduction: (input: VoidRecordInput) => runMutation(() => voidProductionRecord(input), "Produção anulada e retirada dos totais válidos."),
    correctMaintenance: (input: CorrectMaintenanceInput) => runMutation(() => correctBoxMaintenance(input), "Manutenção corrigida e auditada."),
    voidMaintenance: (input: VoidRecordInput) => runMutation(() => voidBoxMaintenance(input), "Manutenção anulada e preservada no histórico."),
    correctEvent: (input: CorrectEventInput) => runMutation(() => correctColonyEvent(input), "Evento corrigido e auditado."),
    voidEvent: (input: VoidRecordInput) => runMutation(() => voidColonyEvent(input), "Evento anulado e preservado na timeline."),
    correctMovementDetails: (input: CorrectMovementDetailsInput) => runMutation(() => correctMovementDetails(input), "Dados descritivos da movimentação corrigidos."),
    voidTransport: (input: VoidRecordInput) => runMutation(() => voidTransport(input), "Transporte anulado sem apagar o registro."),
    updateMovementDocument: (input: UpdateMovementDocumentInput) => runMutation(() => updateMovementDocument(input), "Documento atualizado e auditado."),
    voidMovementDocument: (input: VoidRecordInput) => runMutation(() => voidMovementDocument(input), "Documento invalidado e preservado."),
    correctDivision: (input: CorrectDivisionInput) => runMutation(() => correctColonyDivision(input), "Divisão corrigida e auditada."),
    voidDivision: (input: VoidDivisionInput) => runMutation(() => voidColonyDivision(input), "Divisão anulada de forma controlada."),
    correctOccupancy: (input: CorrectOccupancyInput) => runMutation(() => correctBoxOccupancy(input), "Intervalo de ocupação corrigido e auditado."),
    reverseLifecycle: (input: ReverseRecordInput) => runMutation(() => reverseColonyLifecycle(input), "Operação de ciclo de vida revertida com segurança."),
    reverseMovement: (input: ReverseRecordInput) => runMutation(() => reverseColonyMovement(input), "Transferência revertida com segurança."),
  };

  return { data, stats, recordStates, recordStateMap, connectionStatus, busy, feedback, setFeedback, refresh, actions };
}

export type AppActions = ReturnType<typeof useAppData>["actions"];
export type RecordStateMap = ReturnType<typeof useAppData>["recordStateMap"];
