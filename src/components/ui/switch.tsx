import * as SwitchPrimitive from '@radix-ui/react-switch'
import type { ComponentPropsWithRef } from 'react'
import { cn } from '@/lib/utils'

interface SwitchProps extends ComponentPropsWithRef<typeof SwitchPrimitive.Root> {
  label: string
  tone?: 'default' | 'conflict'
}

export function Switch({
  label,
  tone = 'default',
  className,
  ...props
}: SwitchProps) {
  return (
    <SwitchPrimitive.Root
      aria-label={label}
      className={cn(
        'relative h-[18px] w-[32px] shrink-0 rounded-full border border-line bg-switch-bg transition-colors data-[state=checked]:border-transparent data-[state=checked]:bg-switch-on disabled:opacity-40',
        tone === 'conflict' &&
          'data-[state=checked]:border-danger data-[state=checked]:bg-danger-soft',
        className,
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb
        className={cn(
          'block size-3 translate-x-[2px] rounded-full bg-switch-knob transition-transform data-[state=checked]:translate-x-[16px] data-[state=checked]:bg-switch-knob-on',
          tone === 'conflict' && 'data-[state=checked]:bg-danger',
        )}
      />
    </SwitchPrimitive.Root>
  )
}
