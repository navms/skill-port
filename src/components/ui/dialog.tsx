import * as DialogPrimitive from '@radix-ui/react-dialog'
import { X } from 'lucide-react'
import { useId, type ReactNode } from 'react'
import { useI18n } from '@/i18n'
import { cn } from '@/lib/utils'

interface DialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: ReactNode
  description?: ReactNode
  footer?: ReactNode
  children: ReactNode
  className?: string
  bodyClassName?: string
}

export function Dialog({
  open,
  onOpenChange,
  title,
  description,
  footer,
  children,
  className,
  bodyClassName,
}: DialogProps) {
  const { t } = useI18n()
  const descriptionId = useId()

  return (
    <DialogPrimitive.Root open={open} onOpenChange={onOpenChange}>
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay className="sp-overlay fixed inset-0 z-50 bg-scrim backdrop-blur-[3px]" />
        <DialogPrimitive.Content
          aria-describedby={description ? descriptionId : undefined}
          className={cn(
            'sp-dialog fixed top-1/2 left-1/2 z-50 flex max-h-[84vh] w-[min(520px,94vw)] -translate-x-1/2 -translate-y-1/2 flex-col overflow-hidden rounded-[10px] border border-line-2 bg-panel shadow-[var(--sp-panel-shadow)]',
            className,
          )}
        >
          <div className="flex shrink-0 items-start gap-3 px-5 pt-4">
            <div className="min-w-0 flex-1">
              <DialogPrimitive.Title className="text-base font-semibold tracking-tight text-ink">
                {title}
              </DialogPrimitive.Title>
              {description ? (
                <DialogPrimitive.Description
                  id={descriptionId}
                  className="mt-1 text-xs leading-relaxed text-muted"
                >
                  {description}
                </DialogPrimitive.Description>
              ) : null}
            </div>
            <DialogPrimitive.Close asChild>
              <button
                type="button"
                aria-label={t('common.close')}
                className="flex size-8 shrink-0 items-center justify-center rounded-md border border-line text-muted transition-colors hover:bg-raise hover:text-ink"
              >
                <X size={14} aria-hidden />
              </button>
            </DialogPrimitive.Close>
          </div>

          <div
            className={cn(
              'min-h-0 flex-1 overflow-auto px-5 py-3.5',
              bodyClassName,
            )}
          >
            {children}
          </div>

          {footer ? (
            <div className="flex shrink-0 justify-end gap-2 border-t border-line bg-panel-2 px-5 py-2.5">
              {footer}
            </div>
          ) : null}
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  )
}
