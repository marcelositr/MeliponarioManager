import { useEffect, type ReactNode } from "react";
import { Icon } from "./Icon";

type DialogProps = {
  open: boolean;
  title: string;
  description?: string;
  size?: "small" | "medium" | "large";
  children: ReactNode;
  onClose: () => void;
  closeOnBackdrop?: boolean;
};

export function Dialog({ open, title, description, size = "medium", children, onClose, closeOnBackdrop = false }: DialogProps) {
  useEffect(() => {
    if (!open) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div className="dialog-backdrop" role="presentation" onMouseDown={(event) => {
      if (closeOnBackdrop && event.currentTarget === event.target) onClose();
    }}>
      <section className={`dialog dialog-${size}`} role="dialog" aria-modal="true" aria-labelledby="dialog-title">
        <header className="dialog-header">
          <div>
            <h2 id="dialog-title">{title}</h2>
            {description && <p>{description}</p>}
          </div>
          <button className="icon-button" type="button" onClick={onClose} aria-label="Fechar"><Icon name="close" /></button>
        </header>
        <div className="dialog-body">{children}</div>
      </section>
    </div>
  );
}
