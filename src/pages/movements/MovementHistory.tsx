import { RecordActions } from "../../components/RecordActions";
import { RecordStateBadge } from "../../components/RecordStateBadge";
import type { RecordStateMap } from "../../hooks/useAppData";
import { formatDateTimeBr } from "../../lib/presentation";
import type { TransportReturn } from "../../lib/transport-api";
import type { ColonyMovement } from "../../types";
import { movementLabel } from "./presentation";

type Props = {
  selectedColonyId: string;
  loading: boolean;
  movements: ColonyMovement[];
  returnByMovement: Map<string, TransportReturn>;
  recordStateMap: RecordStateMap;
  busy: boolean;
  transportBusy: boolean;
  hasOpenTransport: boolean;
  onOpenDocuments: (movementId: string) => void;
  onOpenDetail: (item: ColonyMovement) => void;
  onEdit: (item: ColonyMovement) => void;
  onReopen: (item: ColonyMovement) => void;
  onReturn: (item: ColonyMovement) => void;
  onAction: (item: ColonyMovement, mode: "void" | "reverse") => void;
};

export function MovementHistory({
  selectedColonyId,
  loading,
  movements,
  returnByMovement,
  recordStateMap,
  busy,
  transportBusy,
  hasOpenTransport,
  onOpenDocuments,
  onOpenDetail,
  onEdit,
  onReopen,
  onReturn,
  onAction,
}: Props) {
  return <section className="panel wide-list">
    <div className="panel-heading">
      <h2>Histórico da colônia</h2>
      <p>O movimento de saída e o retorno são fatos separados. Reabrir um retorno preserva o registro anterior e a auditoria.</p>
    </div>
    {!selectedColonyId ? <div className="empty-list">Selecione uma colônia na toolbar.</div> : loading ? <div className="empty-list">Carregando...</div> : movements.length === 0 ? <div className="empty-list">Nenhuma movimentação registrada.</div> : <div className="table-wrap">
      <table className="data-table">
        <thead><tr><th>Data</th><th>Tipo</th><th>Origem</th><th>Destino</th><th>Estado</th><th>Ações</th></tr></thead>
        <tbody>{movements.map((item) => {
          const state = recordStateMap.get(`movement:${item.id}`);
          const disabled = Boolean(state?.voidedAt || state?.reversedAt);
          const transportReturn = returnByMovement.get(item.id);
          const secondary = [{ label: "Documentos", onClick: () => onOpenDocuments(item.id) }];
          if (!disabled && item.movementType === "transport") {
            if (transportReturn) {
              if (!hasOpenTransport) secondary.push({ label: "Reabrir transporte…", onClick: () => onReopen(item) });
            } else {
              secondary.push({ label: "Registrar retorno…", onClick: () => onReturn(item) });
              secondary.push({ label: "Anular transporte", onClick: () => onAction(item, "void") });
            }
          } else if (!disabled && item.movementType !== "transport") {
            secondary.push({ label: "Reverter transferência", onClick: () => onAction(item, "reverse") });
          }
          return <tr key={item.id} className={disabled ? "voided-row" : undefined}>
            <td><strong>{formatDateTimeBr(item.movedAt)}</strong></td>
            <td>{movementLabel(item.movementType)}</td>
            <td>{item.fromMeliponaryName}</td>
            <td>{item.toMeliponaryName || item.destination || "—"}</td>
            <td>{item.movementType === "transport" && !disabled
              ? transportReturn
                ? <><span className="badge status-active">Retornado</span><small className="cell-note">{formatDateTimeBr(transportReturn.returnedAt)}</small></>
                : <span className="badge severity-attention">Transporte aberto</span>
              : <RecordStateBadge state={state} />}</td>
            <td><RecordActions busy={busy || transportBusy} onOpen={() => onOpenDetail(item)} onEdit={disabled ? undefined : () => onEdit(item)} secondary={secondary} /></td>
          </tr>;
        })}</tbody>
      </table>
    </div>}
  </section>;
}
