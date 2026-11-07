import { useI18n } from '@/i18n'
import type { SyncResult } from '@/lib/types'
import { cn } from '@/lib/utils'

export function SyncResultView({ result }: { result: SyncResult }) {
  const { t } = useI18n()
  const failures = result.items.filter((item) => !item.ok)

  return (
    <div className="space-y-3">
      <div
        className={cn(
          'rounded-md border px-3 py-2 text-[13px]',
          result.failed > 0 || result.cancelled
            ? 'border-warn-soft bg-warn-soft text-warn'
            : 'border-ok-soft bg-ok-soft text-ok',
        )}
      >
        {result.cancelled
          ? t('sync.cancelled')
          : t('sync.result', {
              applied: result.applied,
              failed: result.failed,
            })}
      </div>
      {failures.length > 0 ? (
        <div className="rounded-md border border-line">
          {failures.map((item) => (
            <div
              key={item.actionId}
              className="border-b border-line px-3 py-2 text-xs text-muted last:border-b-0"
            >
              <span className="font-mono text-[10.5px] text-danger">
                {item.errorCode ?? 'ERROR'}
              </span>
              {item.message ? ` · ${item.message}` : null}
            </div>
          ))}
        </div>
      ) : null}
    </div>
  )
}
