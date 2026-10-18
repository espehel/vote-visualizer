export interface JsonViewerProps {
  value: unknown;
  /** Extra classes, e.g. a max height. */
  className?: string;
}

/** Pretty-printed JSON that scrolls when it overflows. */
export function JsonViewer({ value, className }: JsonViewerProps) {
  return (
    <pre
      className={
        className
          ? `overflow-auto text-xs ${className}`
          : "overflow-auto text-xs"
      }
    >
      {JSON.stringify(value, null, 2)}
    </pre>
  );
}
