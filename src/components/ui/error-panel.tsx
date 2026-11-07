import { AlertTriangle } from 'lucide-react'
import { Button } from './button'

interface ErrorPanelProps {
  title: string
  message?: string
  retryLabel: string
  onRetry: () => void
}

export function ErrorPanel({
  title,
  message,
  retryLabel,
  onRetry,
}: ErrorPanelProps) {
  return (
    <div
      role="alert"
      className="flex flex-col items-center justify-center px-6 py-14 text-center"
    >
      <div className="mb-3 flex size-10 items-center justify-center rounded-lg border border-danger-soft bg-danger-soft text-danger">
        <AlertTriangle size={18} aria-hidden />
      </div>
      <p className="text-sm font-medium text-ink-2">{title}</p>
      {message ? (
        <p className="mt-1 max-w-sm text-[13px] leading-relaxed text-muted">
          {message}
        </p>
      ) : null}
      <Button variant="secondary" size="sm" onClick={onRetry} className="mt-4">
        {retryLabel}
      </Button>
    </div>
  )
}
