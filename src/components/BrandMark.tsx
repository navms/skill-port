export function BrandMark({
  size = 18,
  className,
}: {
  size?: number
  className?: string
}) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 18 18"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.4"
      strokeLinecap="round"
      aria-hidden
      className={className}
    >
      <circle cx="3.2" cy="9" r="1.5" fill="currentColor" stroke="none" />
      <path d="M4.8 9h2.1c2.6 0 2.6-4.2 5.2-4.2M6.9 9c2.6 0 2.6 4.2 5.2 4.2" />
      <circle cx="14.3" cy="4.8" r="1.4" />
      <circle cx="14.3" cy="13.2" r="1.4" />
    </svg>
  )
}
