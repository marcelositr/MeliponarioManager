import type { Alert } from "./types";

export type DashboardCount = { label: string; count: number };
export type RecentProduction = { colonyCode: string; productType: string; quantity: number; unit: string; harvestedAt: string };
export type RecentMovement = { colonyCode: string; movementType: string; movedAt: string; destination?: string | null };
export type DashboardOverview = {
  colonyStatuses: DashboardCount[];
  speciesDistribution: DashboardCount[];
  inspectionStrengths: DashboardCount[];
  occupiedBoxes: number;
  freeBoxes: number;
  alerts: Alert[];
  recentProduction: RecentProduction[];
  recentMovements: RecentMovement[];
};
