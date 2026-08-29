import { useEffect, useId, useRef, type ReactNode } from "react";
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

const focusableSelector = [
  "button:not([disabled])",
  "input:not([disabled])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  "a[href]",
  "[tabindex]:not([tabindex='-1'])",
].join(",");

export function Dialog({ open, title, description, size = "medium", children, onClose, closeOnBackdrop = false }: DialogProps) {
  const titleId = useId();
  const descriptionId = useId();
  const backdropRef = useRef<HTMLDivElement>(null);
  const dialogRef = useRef<HTMLElement>(null);

  useEffect(() => {
    if (!open) return;
    const previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const frame = window.requestAnimationFrame(() => {
      const dialog = dialogRef.current;
      if (!dialog || !isTopDialog(backdropRef.current)) return;
      const preferred = dialog.querySelector<HTMLElement>("[autofocus]");
      const first = getFocusable(dialog)[0];
      (preferred ?? first ?? dialog).focus();
    });

    const onKeyDown = (event: KeyboardEvent) => {
      const dialog = dialogRef.current;
      if (!dialog || !isTopDialog(backdropRef.current)) return;
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopImmediatePropagation();
        onClose();
        return;
      }
      if (event.key !== "Tab") return;
      const focusable = getFocusable(dialog);
      if (focusable.length === 0) {
        event.preventDefault();
        dialog.focus();
        return;
      }
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.cancelAnimationFrame(frame);
      window.removeEventListener("keydown", onKeyDown);
      if (isVisible(previousFocus)) previousFocus.focus();
    };
  }, [open, onClose]);

  if (!open) return null;

  return <div ref={backdropRef} className="dialog-backdrop" role="presentation" onMouseDown={(event) => {
    if (closeOnBackdrop && event.currentTarget === event.target && isTopDialog(backdropRef.current)) onClose();
  }}>
    <section ref={dialogRef} tabIndex={-1} className={`dialog dialog-${size}`} role="dialog" aria-modal="true" aria-labelledby={titleId} aria-describedby={description ? descriptionId : undefined}>
      <header className="dialog-header">
        <div><h2 id={titleId}>{title}</h2>{description && <p id={descriptionId}>{description}</p>}</div>
        <button className="icon-button" type="button" onClick={onClose} aria-label="Fechar"><Icon name="close" /></button>
      </header>
      <div className="dialog-body">{children}</div>
    </section>
  </div>;
}

function getFocusable(root: HTMLElement) {
  return Array.from(root.querySelectorAll<HTMLElement>(focusableSelector)).filter(isVisible);
}

function isVisible(element: HTMLElement | null): element is HTMLElement {
  return Boolean(element?.isConnected && !element.hidden && element.getAttribute("aria-hidden") !== "true" && element.getClientRects().length > 0);
}

function isTopDialog(backdrop: HTMLDivElement | null) {
  if (!backdrop) return false;
  const dialogs = document.querySelectorAll<HTMLDivElement>(".dialog-backdrop");
  return dialogs.length > 0 && dialogs[dialogs.length - 1] === backdrop;
}
