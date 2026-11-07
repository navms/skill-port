import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'

interface FieldProps {
  label: string
  htmlFor?: string
  hint?: ReactNode
  error?: string | null
  className?: string
  children: ReactNode
}

export function Field({
  label,
  htmlFor,
  hint,
  error,
  className,
  children,
}: FieldProps) {
  return (
    <div className={cn('space-y-1.5', className)}>
      <label
        htmlFor={htmlFor}
        className="block text-xs font-medium tracking-wide text-muted"
      >
        {label}
      </label>
      {children}
      {error ? (
        <p role="alert" className="text-xs leading-relaxed text-danger">
          {error}
        </p>
      ) : hint ? (
        <p className="text-xs leading-relaxed text-faint">{hint}</p>
      ) : null}
    </div>
  )
}
