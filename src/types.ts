export type View = "dashboard" | "meliponaries" | "species" | "colonies" | "boxes" | "inspections";

export type CoreSummary = {
  meliponaries: number;
  species: number;
  colonies: number;
  boxes: number;
};

export type Meliponary = {
  id: string;
  name: string;
  responsibleName?: string | null;
  location?: string | null;
  notes?: string | null;
  createdAt: string;
};

export type Species = {
  id: string;
  commonName: string;
  scientificName?: string | null;
  genus?: string | null;
  notes?: string | null;
  createdAt: string;
};

export type HiveBox = {
  id: string;
  meliponaryId: string;
  code: string;
  model?: string | null;
  material?: string | null;
  locationNote?: string | null;
  status: string;
  notes?: string | null;
  currentColonyCode?: string | null;
  createdAt: string;
};

export type Colony = {
  id: string;
  meliponaryId: string;
  speciesId: string;
  code: string;
  originType: string;
  originNotes?: string | null;
  installedAt?: string | null;
  status: string;
  motherColonyId?: string | null;
  notes?: string | null;
  currentBoxCode?: string | null;
  createdAt: string;
};

export type Inspection = {
  id: string;
  colonyId: string;
  colonyCode: string;
  boxId?: string | null;
  boxCode?: string | null;
  inspectedAt: string;
  strength: string;
  queenPresent?: boolean | null;
  layingStatus?: string | null;
  foodReserves?: string | null;
  broodStatus?: string | null;
  pestsNotes?: string | null;
  observations?: string | null;
  actionsTaken?: string | null;
  nextInspectionAt?: string | null;
  createdAt: string;
};

export type DashboardStats = CoreSummary & {
  inspections: number;
  photos: number;
  events: number;
  divisions: number;
  feedings: number;
  production: number;
  movements: number;
  documents: number;
  maintenance: number;
  lifecycle: number;
  alerts: number;
};

export type CoreData = {
  meliponaries: Meliponary[];
  species: Species[];
  colonies: Colony[];
  boxes: HiveBox[];
};

export type CreateMeliponaryInput = {
  name: string;
  responsibleName?: string;
  location?: string;
  notes?: string;
};

export type CreateSpeciesInput = {
  commonName: string;
  scientificName?: string;
  genus?: string;
  notes?: string;
};

export type CreateBoxInput = {
  meliponaryId: string;
  code: string;
  model?: string;
  material?: string;
  locationNote?: string;
  notes?: string;
};

export type CreateColonyInput = {
  meliponaryId: string;
  speciesId: string;
  code: string;
  originType?: string;
  originNotes?: string;
  installedAt?: string;
  motherColonyId?: string;
  notes?: string;
};

export type PlaceColonyInput = {
  colonyId: string;
  boxId: string;
  startedAt?: string;
  reason?: string;
  notes?: string;
};

export type CreateInspectionInput = {
  colonyId: string;
  inspectedAt?: string;
  strength?: string;
  queenPresent?: boolean | null;
  layingStatus?: string;
  foodReserves?: string;
  broodStatus?: string;
  pestsNotes?: string;
  observations?: string;
  actionsTaken?: string;
  nextInspectionAt?: string;
};
