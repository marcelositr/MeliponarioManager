import { Children, cloneElement, isValidElement, useEffect, useId, useRef, type ReactElement, type ReactNode } from "react";
import { Icon } from "./Icon";

type DialogProps = {
  open: boolean;
  title: string;
  description?: string;
  size?: "small" | "medium" | "large";
  children: ReactNode;
  footer?: ReactNode;
  onClose: () => void;
  closeOnBackdrop?: boolean;
};

type ElementProps = {
  children?: ReactNode;
  className?: string;
  id?: string;
  type?: string;
  form?: string;
};

const focusableSelector = [
  "button:not([disabled])",
  "input:not([disabled])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  "a[href]",
  "[tabindex]:not([tabindex='-1'])",
].join(",");

export function Dialog({ open, title, description, size = "medium", children, footer, onClose, closeOnBackdrop = false }: DialogProps) {
  const titleId = useId();
  const descriptionId = useId();
  const backdropRef = useRef<HTMLDivElement>(null);
  const dialogRef = useRef<HTMLElement>(null);
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;
  const { body, promotedFooter } = splitDialogActions(children, titleId);
  const footerContent = footer || promotedFooter;

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
        onCloseRef.current();
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
  }, [open]);

  if (!open) return null;

  return <div ref={backdropRef} className="dialog-backdrop" role="presentation" onMouseDown={(event) => {
    if (closeOnBackdrop && event.currentTarget === event.target && isTopDialog(backdropRef.current)) onCloseRef.current();
  }}>
    <section ref={dialogRef} tabIndex={-1} className={`dialog dialog-${size}`} role="dialog" aria-modal="true" aria-labelledby={titleId} aria-describedby={description ? descriptionId : undefined}>
      <header className="dialog-header">
        <div><h2 id={titleId}>{title}</h2>{description && <p id={descriptionId}>{description}</p>}</div>
        <button className="icon-button" type="button" onClick={() => onCloseRef.current()} aria-label="Fechar"><Icon name="close" /></button>
      </header>
      <div className="dialog-body">{body}</div>
      {footerContent && <footer className="dialog-footer">{footerContent}</footer>}
    </section>
  </div>;
}

function splitDialogActions(children: ReactNode, idPrefix: string) {
  const actions: ReactNode[] = [];
  let formIndex = 0;

  function walk(node: ReactNode, directFormId?: string): ReactNode {
    return Children.map(node, (child) => {
      if (!isValidElement(child)) return child;
      const element = child as ReactElement<ElementProps>;
      const props = element.props;
      const className = props.className || "";

      if (directFormId && className.split(/\s+/).includes("form-actions")) {
        actions.push(bindSubmitButtons(element, directFormId));
        return null;
      }

      const isForm = element.type === "form";
      const formId = isForm ? (props.id || `${idPrefix}-form-${formIndex++}`) : undefined;
      const nextChildren = props.children === undefined
        ? undefined
        : isForm
          ? walk(props.children, formId)
          : walk(props.children);

      if (isForm) return cloneElement(element, { id: formId }, nextChildren);
      if (props.children !== undefined) return cloneElement(element, undefined, nextChildren);
      return element;
    });
  }

  return {
    body: walk(children),
    promotedFooter: actions.length > 0 ? <>{actions}</> : undefined,
  };
}

function bindSubmitButtons(node: ReactNode, formId: string): ReactNode {
  return Children.map(node, (child) => {
    if (!isValidElement(child)) return child;
    const element = child as ReactElement<ElementProps>;
    const props = element.props;
    const nextChildren = props.children === undefined ? undefined : bindSubmitButtons(props.children, formId);
    const form = element.type === "button" && props.type === "submit" ? formId : props.form;
    return cloneElement(element, form ? { form } : undefined, nextChildren);
  });
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
