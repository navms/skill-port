import { useId, type ComponentPropsWithRef, type ReactNode } from 'react'
import { cn } from '@/lib/utils'

interface CheckboxProps extends Omit<ComponentPropsWithRef<'input'>, 'type'> {
  label: ReactNode
}

export function Checkbox({
  label,
  id,
  className,
  ...props
}: CheckboxProps) {
  const generatedId = useId()
  const inputId = id ?? generatedId

  return (
    <label
      htmlFor={inputId}
      className={cn(
        'flex cursor-pointer items-start gap-2 text-[12.5px] leading-relaxed text-ink-2',
        className,
      )}
    >
      <input
        id={inputId}
        type="checkbox"
        className="mt-0.5 size-4 shrink-0 accent-[var(--sp-accent)]"
        {...props}
      />
      <span>{label}</span>
    </label>
  )
}
