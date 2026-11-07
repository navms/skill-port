import type { ReactNode } from 'react'
import { TONE_DOT, TONE_SOFT, TONE_TEXT, type DisplayState, toneForState } from '@/lib/status'
import type { Tone } from '@/lib/types'
import { cn } from '@/lib/utils'

export function StatusDot({
  tone,
  className,
}: {
  tone: Tone
  className?: string
}) {
  return (
    <span
      aria-hidden
      className={cn('inline-block size-1.5 shrink-0 rounded-full', TONE_DOT[tone], className)}
    />
  )
}

export function StateDot({
  state,
  className,
}: {
  state: DisplayState
  className?: string
}) {
  return <StatusDot tone={toneForState(state)} className={className} />
}

export function ToneText({
  tone,
  children,
  className,
}: {
  tone: Tone
  children: ReactNode
  className?: string
}) {
  return <span className={cn(TONE_TEXT[tone], className)}>{children}</span>
}

export function Tag({
  tone = 'muted',
  mono = false,
  className,
  children,
}: {
  tone?: Tone
  mono?: boolean
  className?: string
  children: ReactNode
}) {
  return (
    <span
      className={cn(
        'inline-flex h-6 items-center gap-1 rounded border px-1.5 text-xs whitespace-nowrap',
        mono && 'font-mono text-[11px] tracking-tight',
        tone === 'muted' ? 'border-line text-muted' : cn(TONE_TEXT[tone], TONE_SOFT[tone]),
        className,
      )}
    >
      {children}
    </span>
  )
}
