export function Gradient({
  conic,
  className,
  small,
}: {
  small?: boolean;
  conic?: boolean;
  className?: string;
}) {
  return (
    <span
      className={`absolute rounded-[100%] mix-blend-normal will-change-[filter] ${
        small ? "blur-[32px]" : "blur-[75px]"
      } ${
        conic
          ? "bg-gradient-to-r bg-red-1000 from-10% via-30% via-purple-1000 to-100% to-blue-1000"
          : ""
      } ${className ?? ""}`}
    />
  );
}
