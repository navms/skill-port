import { AlertTriangle, Check, X, XCircle } from 'lucide-react'
import { useI18n } from '@/i18n'
import { cn } from '@/lib/utils'
import { useToastStore, type ToastTone } from './toast-store'

const ICONS: Record<ToastTone, typeof Check> = {
  ok: Check,
  warn: AlertTriangle,
  err: XCircle,
}

const BORDER: Record<ToastTone, string> = {
  ok: 'border-l-accent',
  warn: 'border-l-warn',
  err: 'border-l-danger',
}

const ICON_COLOR: Record<ToastTone, string> = {
  ok: 'text-ok',
  warn: 'text-warn',
  err: 'text-danger',
}

export function Toaster() {
  const items = useToastStore((state) => state.items)
  const dismiss = useToastStore((state) => state.dismiss)
  const { t } = useI18n()

  return (
    <div
      aria-live="polite"
      className="pointer-events-none fixed inset-x-0 bottom-11 z-[100] flex flex-col items-center gap-2 px-4"
    >
      {items.map((item) => {
        const Icon = ICONS[item.tone]
        return (
          <div
            key={item.id}
            role={item.tone === 'err' ? 'alert' : 'status'}
            className={cn(
              'sp-toast pointer-events-auto flex max-w-[min(540px,92vw)] min-w-[260px] items-center gap-2.5 rounded-md border border-line-2 border-l-2 bg-toast-bg px-3 py-2.5 text-[13px] text-toast-ink shadow-xl',
              BORDER[item.tone],
            )}
          >
            <Icon
              size={14}
              aria-hidden
              className={cn('shrink-0', ICON_COLOR[item.tone])}
            />
            <span className="min-w-0 flex-1 leading-snug">{item.message}</span>
            {item.action ? (
              <button
                type="button"
                onClick={() => {
                  item.action?.onClick()
                  dismiss(item.id)
                }}
                className="shrink-0 rounded px-1.5 py-0.5 text-xs font-medium text-toast-ink underline underline-offset-2 hover:bg-white/10"
              >
                {item.action.label}
              </button>
            ) : null}
            <button
              type="button"
              onClick={() => dismiss(item.id)}
              aria-label={t('common.close')}
              className="shrink-0 rounded p-0.5 text-toast-ink/60 hover:text-toast-ink"
            >
              <X size={13} aria-hidden />
            </button>
          </div>
        )
      })}
    </div>
  )
}
