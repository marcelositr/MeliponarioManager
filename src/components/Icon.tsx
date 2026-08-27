import type { SVGProps } from "react";

export type IconName =
  | "dashboard" | "alerts" | "meliponary" | "colony" | "box" | "species"
  | "inspection" | "feeding" | "production" | "maintenance" | "history"
  | "genealogy" | "movement" | "lifecycle" | "data" | "menu" | "chevron"
  | "refresh" | "plus" | "close" | "sun" | "moon" | "system" | "database"
  | "check" | "warning" | "info" | "more";

type IconProps = SVGProps<SVGSVGElement> & { name: IconName };

const paths: Record<IconName, React.ReactNode> = {
  dashboard: <><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></>,
  alerts: <><path d="M18 8a6 6 0 0 0-12 0c0 7-3 7-3 9h18c0-2-3-2-3-9"/><path d="M10 21h4"/></>,
  meliponary: <><path d="M3 21h18"/><path d="M5 21V7l7-4 7 4v14"/><path d="M9 21v-6h6v6"/></>,
  colony: <><path d="M12 3 4 7.5v9L12 21l8-4.5v-9L12 3Z"/><path d="m4 7.5 8 4.5 8-4.5"/><path d="M12 12v9"/></>,
  box: <><path d="M4 5h16v14H4z"/><path d="M4 9h16"/><path d="M9 13h6"/></>,
  species: <><path d="M12 21c4-4 7-8 7-12a7 7 0 0 0-14 0c0 4 3 8 7 12Z"/><path d="M9 9c1-2 3-3 6-3-1 3-2 5-5 6"/></>,
  inspection: <><rect x="5" y="4" width="14" height="17" rx="2"/><path d="M9 4V2h6v2"/><path d="m8 12 2 2 5-5"/></>,
  feeding: <><path d="M5 21c0-6 2-10 7-13 5 3 7 7 7 13"/><path d="M8 14h8"/><path d="M9 18h6"/></>,
  production: <><path d="M4 21V9l8-5 8 5v12"/><path d="M8 21v-7h8v7"/><path d="M7 9h10"/></>,
  maintenance: <><path d="m14 6 4-4 4 4-4 4"/><path d="M18 2 9 11"/><path d="m10 14-6 6"/><circle cx="7" cy="17" r="3"/></>,
  history: <><path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5"/><path d="M12 7v5l3 2"/></>,
  genealogy: <><circle cx="12" cy="5" r="2"/><circle cx="6" cy="19" r="2"/><circle cx="18" cy="19" r="2"/><path d="M12 7v5M6 17v-3h12v3"/></>,
  movement: <><path d="M5 7h12"/><path d="m14 4 3 3-3 3"/><path d="M19 17H7"/><path d="m10 14-3 3 3 3"/></>,
  lifecycle: <><path d="M20 12a8 8 0 0 1-14 5"/><path d="M4 12A8 8 0 0 1 18 7"/><path d="m18 3 1 4-4 1"/><path d="m6 21-1-4 4-1"/></>,
  data: <><ellipse cx="12" cy="5" rx="8" ry="3"/><path d="M4 5v6c0 1.7 3.6 3 8 3s8-1.3 8-3V5"/><path d="M4 11v6c0 1.7 3.6 3 8 3s8-1.3 8-3v-6"/></>,
  menu: <><path d="M4 7h16M4 12h16M4 17h16"/></>,
  chevron: <path d="m9 18 6-6-6-6"/>,
  refresh: <><path d="M20 6v5h-5"/><path d="M4 18v-5h5"/><path d="M6.1 9A7 7 0 0 1 18.5 6.5L20 8M4 16l1.5 1.5A7 7 0 0 0 17.9 15"/></>,
  plus: <path d="M12 5v14M5 12h14"/>,
  close: <path d="m6 6 12 12M18 6 6 18"/>,
  sun: <><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/></>,
  moon: <path d="M20 15.3A8 8 0 1 1 8.7 4 6.5 6.5 0 0 0 20 15.3Z"/>,
  system: <><rect x="3" y="4" width="18" height="13" rx="2"/><path d="M8 21h8M12 17v4"/></>,
  database: <><ellipse cx="12" cy="5" rx="8" ry="3"/><path d="M4 5v7c0 1.7 3.6 3 8 3s8-1.3 8-3V5M4 12v7"/></>,
  check: <path d="m5 12 4 4L19 6"/>,
  warning: <><path d="M12 3 2.5 20h19L12 3Z"/><path d="M12 9v4M12 17h.01"/></>,
  info: <><circle cx="12" cy="12" r="9"/><path d="M12 11v6M12 7h.01"/></>,
  more: <><circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/></>,
};

export function Icon({ name, ...props }: IconProps) {
  return <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" focusable="false" {...props}>{paths[name]}</svg>;
}
