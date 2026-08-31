export function normalizeDateTime(value?: string) {
  if (!value) return undefined;
  const normalized = value.replace("T", " ");
  return normalized.length === 16 ? `${normalized}:00` : normalized;
}

export function toInputDateTime(value: string) {
  return value.replace(" ", "T").slice(0, 16);
}

export function movementLabel(value: string) {
  return value === "internal_transfer"
    ? "Transferência interna"
    : value === "external_transfer"
      ? "Transferência externa"
      : "Transporte temporário";
}

export function documentLabel(value: string) {
  const labels: Record<string, string> = {
    gta: "GTA",
    authorization: "Autorização",
    invoice: "Nota fiscal",
    receipt: "Recibo",
    declaration: "Declaração",
    protocol: "Protocolo",
    certificate: "Certificado",
    other: "Outro",
  };
  return labels[value] || value;
}

export function DocumentTypeOptions() {
  return <>
    <option value="gta">GTA</option>
    <option value="authorization">Autorização</option>
    <option value="invoice">Nota fiscal</option>
    <option value="receipt">Recibo</option>
    <option value="declaration">Declaração</option>
    <option value="protocol">Protocolo</option>
    <option value="certificate">Certificado</option>
    <option value="other">Outro</option>
  </>;
}
