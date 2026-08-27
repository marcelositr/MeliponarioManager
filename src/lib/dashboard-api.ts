import { invoke } from "@tauri-apps/api/core";
import type { DashboardOverview } from "../dashboard-types";

export const loadDashboardOverview = () => invoke<DashboardOverview>("get_dashboard_overview");
