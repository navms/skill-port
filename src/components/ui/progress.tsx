import { cn } from '@/lib/utils'

interface ProgressProps {
  value: number
  max: number
  label: string
  className?: string
}

export function Progress({ value, max, label, className }: ProgressProps) {
  const percentage = max > 0 ? Math.min(100, Math.round((value / max) * 100)) : 0
  return (
    <div
      role="progressbar"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={max}
      aria-valuenow={value}
      className={cn(
        'h-1.5 overflow-hidden rounded-full bg-panel-2',
        className,
      )}
    >
      <div
        className="h-full rounded-full bg-accent transition-[width] duration-200"
        style={{ width: `${percentage}%` }}
      />
    </div>
  )
}
