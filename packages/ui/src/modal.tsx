"use client";

import { type ReactNode, useEffect, useRef } from "react";

export interface ModalProps {
  open: boolean;
  /** Called when the user closes the modal (Esc, close button, or backdrop). */
  onClose: () => void;
  /** Extra classes for the modal box, e.g. a width. */
  className?: string;
  children: ReactNode;
}

/** A daisyUI modal on a native `<dialog>`, controlled by `open`. */
export function Modal({ open, onClose, className, children }: ModalProps) {
  const ref = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = ref.current;
    if (!dialog) return;
    if (open && !dialog.open) dialog.showModal();
    if (!open && dialog.open) dialog.close();
  }, [open]);

  return (
    <dialog ref={ref} className="modal" onClose={onClose}>
      <div className={className ? `modal-box ${className}` : "modal-box"}>
        <form method="dialog">
          <button
            type="submit"
            aria-label="Close"
            className="btn btn-sm btn-circle btn-ghost absolute top-2 right-2"
          >
            ✕
          </button>
        </form>
        {children}
      </div>
      <form method="dialog" className="modal-backdrop">
        <button type="submit">close</button>
      </form>
    </dialog>
  );
}
