import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'

export function Kbd({
  children,
  className,
}: {
  children: ReactNode
  className?: string
}) {
  return (
    <kbd
      className={cn(
        'inline-flex h-5 items-center rounded border border-line-2 border-b-2 bg-panel-2 px-1.5 text-[11px] leading-none text-muted',
        className,
      )}
    >
      {children}
    </kbd>
  )
}
