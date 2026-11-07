import type { ComponentPropsWithRef } from 'react'
import { cn } from '@/lib/utils'

const FIELD_CLASS =
  'w-full rounded-md border border-line-2 bg-panel px-2.5 text-ink placeholder:text-faint transition-colors hover:border-line-2 focus:border-accent focus:outline-none disabled:opacity-50'

export function Input({
  className,
  ...props
}: ComponentPropsWithRef<'input'>) {
  return <input className={cn(FIELD_CLASS, 'h-9 text-[13px]', className)} {...props} />
}

export function Textarea({
  className,
  ...props
}: ComponentPropsWithRef<'textarea'>) {
  return (
    <textarea
      className={cn(
        FIELD_CLASS,
        'min-h-24 resize-y py-2 font-mono text-xs leading-relaxed',
        className,
      )}
      {...props}
    />
  )
}
