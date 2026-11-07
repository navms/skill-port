import type { ButtonHTMLAttributes } from 'react'
import { cn } from '@/lib/utils'

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger'
export type ButtonSize = 'sm' | 'md' | 'icon'

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant
  size?: ButtonSize
}

const VARIANTS: Record<ButtonVariant, string> = {
  primary:
    'border-accent bg-accent text-accent-ink hover:brightness-105 active:brightness-95',
  secondary:
    'border-line-2 bg-transparent text-ink-2 hover:bg-raise hover:text-ink',
  ghost:
    'border-transparent bg-transparent text-muted hover:bg-panel-2 hover:text-ink',
  danger:
    'border-danger bg-danger text-bg hover:brightness-105 active:brightness-95',
}

const SIZES: Record<ButtonSize, string> = {
  sm: 'h-8 gap-1.5 px-2.5 text-xs',
  md: 'h-9 gap-2 px-3 text-[13px]',
  icon: 'size-8 p-0',
}

export function Button({
  variant = 'secondary',
  size = 'md',
  className,
  type = 'button',
  ...props
}: ButtonProps) {
  return (
    <button
      type={type}
      className={cn(
        'inline-flex shrink-0 items-center justify-center rounded-md border font-medium whitespace-nowrap transition-[background-color,color,border-color,filter] disabled:pointer-events-none disabled:opacity-45',
        VARIANTS[variant],
        SIZES[size],
        className,
      )}
      {...props}
    />
  )
}
