import { Loader2 } from 'lucide-react'
import { cn } from '@/lib/utils'

export function Spinner({ className }: { className?: string }) {
  return (
    <Loader2
      size={14}
      aria-hidden
      className={cn('animate-spin text-muted', className)}
    />
  )
}

export function LoadingBlock({ label }: { label: string }) {
  return (
    <div className="flex items-center justify-center gap-2 px-6 py-14 text-[13px] text-muted">
      <Spinner />
      <span>{label}</span>
    </div>
  )
}
