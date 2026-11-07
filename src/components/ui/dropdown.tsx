import * as DropdownPrimitive from '@radix-ui/react-dropdown-menu'
import type { ComponentPropsWithRef, ReactNode } from 'react'
import { cn } from '@/lib/utils'

interface DropdownMenuProps {
  trigger: ReactNode
  children: ReactNode
  align?: 'start' | 'center' | 'end'
  className?: string
}

export function DropdownMenu({
  trigger,
  children,
  align = 'end',
  className,
}: DropdownMenuProps) {
  return (
    <DropdownPrimitive.Root>
      <DropdownPrimitive.Trigger asChild>{trigger}</DropdownPrimitive.Trigger>
      <DropdownPrimitive.Portal>
        <DropdownPrimitive.Content
          align={align}
          sideOffset={6}
          className={cn(
            'z-[70] max-h-[min(420px,70vh)] min-w-[200px] overflow-auto rounded-lg border border-line-2 bg-panel p-1 shadow-[var(--sp-menu-shadow)]',
            className,
          )}
        >
          {children}
        </DropdownPrimitive.Content>
      </DropdownPrimitive.Portal>
    </DropdownPrimitive.Root>
  )
}

interface DropdownItemProps
  extends ComponentPropsWithRef<typeof DropdownPrimitive.Item> {
  icon?: ReactNode
  danger?: boolean
}

export function DropdownItem({
  icon,
  danger = false,
  className,
  children,
  ...props
}: DropdownItemProps) {
  return (
    <DropdownPrimitive.Item
      className={cn(
        'flex cursor-default select-none items-center gap-2 rounded-[5px] px-2 py-1.5 text-[12.5px] text-ink-2 outline-none data-[disabled]:pointer-events-none data-[disabled]:opacity-40 data-[highlighted]:bg-panel-2 data-[highlighted]:text-ink',
        danger &&
          'text-danger data-[highlighted]:bg-danger-soft data-[highlighted]:text-danger',
        className,
      )}
      {...props}
    >
      {icon ? (
        <span className="flex size-4 shrink-0 items-center justify-center text-current">
          {icon}
        </span>
      ) : null}
      {children}
    </DropdownPrimitive.Item>
  )
}

export function DropdownSeparator() {
  return <DropdownPrimitive.Separator className="my-1 h-px bg-line" />
}

export function DropdownLabel({ children }: { children: ReactNode }) {
  return (
    <DropdownPrimitive.Label className="px-2 pt-1.5 pb-1 font-mono text-[10px] tracking-[0.12em] text-faint uppercase">
      {children}
    </DropdownPrimitive.Label>
  )
}

export function DropdownCheckboxItem({
  checked,
  onCheckedChange,
  children,
}: {
  checked: boolean
  onCheckedChange: (checked: boolean) => void
  children: ReactNode
}) {
  return (
    <DropdownPrimitive.CheckboxItem
      checked={checked}
      onCheckedChange={onCheckedChange}
      className="flex cursor-default select-none items-center gap-2 rounded-[5px] px-2 py-1.5 text-[12.5px] text-ink-2 outline-none data-[highlighted]:bg-panel-2 data-[highlighted]:text-ink"
    >
      <span className="flex size-4 items-center justify-center">
        <DropdownPrimitive.ItemIndicator>
          <span className="size-1.5 rounded-full bg-ok" />
        </DropdownPrimitive.ItemIndicator>
      </span>
      {children}
    </DropdownPrimitive.CheckboxItem>
  )
}
