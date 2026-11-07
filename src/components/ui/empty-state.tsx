import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'

interface EmptyStateProps {
  icon?: ReactNode
  title: string
  body?: string
  actions?: ReactNode
  className?: string
}

export function EmptyState({
  icon,
  title,
  body,
  actions,
  className,
}: EmptyStateProps) {
  return (
    <div
      className={cn(
        'flex flex-col items-center justify-center px-6 py-16 text-center',
        className,
      )}
    >
      {icon ? (
        <div className="mb-3 flex size-10 items-center justify-center rounded-lg border border-line bg-panel-2 text-muted">
          {icon}
        </div>
      ) : null}
      <p className="text-sm font-medium text-ink-2">{title}</p>
      {body ? (
        <p className="mt-1 max-w-md text-[13px] leading-relaxed text-muted">
          {body}
        </p>
      ) : null}
      {actions ? <div className="mt-4 flex gap-2">{actions}</div> : null}
    </div>
  )
}
