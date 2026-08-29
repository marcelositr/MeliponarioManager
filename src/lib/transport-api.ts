import { invoke } from "@tauri-apps/api/core";

export type TransportReturn = {
  id: string;
  movementId: string;
  returnedAt: string;
  notes?: string | null;
  reversedAt?: string | null;
  reversalReason?: string | null;
  createdAt: string;
};

export type CompleteTransportInput = {
  movementId: string;
  returnedAt?: string;
  notes?: string;
};

export type ReopenTransportInput = {
  movementId: string;
  reason: string;
};

export const completeTransport = (input: CompleteTransportInput) =>
  invoke<TransportReturn>("complete_transport", { input });

export const listTransportReturns = (colonyId: string) =>
  invoke<TransportReturn[]>("list_transport_returns", { colonyId });

export const reopenTransport = (input: ReopenTransportInput) =>
  invoke<void>("reopen_transport", { input });
