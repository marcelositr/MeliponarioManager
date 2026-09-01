import { RecordActions } from "../../components/RecordActions";
import { RecordStateBadge } from "../../components/RecordStateBadge";
import type { RecordStateMap } from "../../hooks/useAppData";
import { formatDateTimeBr } from "../../lib/presentation";
import type { BoxMaintenance, HiveBox } from "../../types";
import { maintenanceLabel } from "./presentation";

type Props = {
  boxes: HiveBox[];
  selectedBoxId: string;
  maintenance: BoxMaintenance[];
  loading: boolean;
  error: string;
  busy: boolean;
  recordStateMap: RecordStateMap;
  onSelectBox: (boxId: string) => void;
  onOpen: (item: BoxMaintenance) => void;
  onEdit: (item: BoxMaintenance) => void;
  onVoid: (item: BoxMaintenance) => void;
};

export function AssetsMaintenancePanel({ boxes, selectedBoxId, maintenance, loading, error, busy, recordStateMap, onSelectBox, onOpen, onEdit, onVoid }: Props) {
  return <section className="panel wide-list">
    <div className="panel-heading"><h2>Histórico da caixa</h2><p>Correções revalidam data, caixa e próxima manutenção; anulações não apagam o registro.</p></div>
    <label className="field"><span>Caixa</span><select value={selectedBoxId} onChange={(event) => onSelectBox(event.target.value)}><option value="">Selecione...</option>{boxes.map((box) => <option value={box.id} key={box.id}>{box.code} {box.currentColonyCode ? `· ${box.currentColonyCode}` : "· vazia"}</option>)}</select></label>
    {!selectedBoxId ? <div className="empty-list section-gap">Selecione uma caixa para consultar o histórico.</div> : error ? <div className="inline-notice error section-gap" role="alert">{error}</div> : loading ? <div className="empty-list section-gap" role="status">Carregando manutenções...</div> : maintenance.length === 0 ? <div className="empty-list section-gap">Nenhuma manutenção registrada.</div> : <div className="table-wrap section-gap"><table className="data-table"><thead><tr><th>Data</th><th>Tipo</th><th>Colônia</th><th>Próxima</th><th>Estado</th><th>Ações</th></tr></thead><tbody>{maintenance.map((item) => {
      const state = recordStateMap.get(`box_maintenance:${item.id}`);
      return <tr key={item.id} className={state?.voidedAt ? "voided-row" : undefined}>
        <td><strong>{formatDateTimeBr(item.maintainedAt)}</strong></td>
        <td>{maintenanceLabel(item.maintenanceType)}</td>
        <td>{item.colonyCode || "Caixa vazia"}</td>
        <td>{item.nextMaintenanceAt ? formatDateTimeBr(item.nextMaintenanceAt) : "—"}</td>
        <td><RecordStateBadge state={state} /></td>
        <td><RecordActions busy={busy} onOpen={() => onOpen(item)} onEdit={state?.voidedAt ? undefined : () => onEdit(item)} secondary={[{ label: "Anular", onClick: () => onVoid(item), disabled: Boolean(state?.voidedAt), danger: true }]} /></td>
      </tr>;
    })}</tbody></table></div>}
  </section>;
}
