import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Check, FolderOpen, X } from 'lucide-react'
import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Dialog } from '@/components/ui/dialog'
import { Field } from '@/components/ui/field'
import { Textarea } from '@/components/ui/input'
import { toast } from '@/components/ui/toast-store'
import { useI18n } from '@/i18n'
import { api, isTauri } from '@/lib/api'
import { toErrorMessage } from '@/lib/errors'
import { invalidateLibrary } from '@/lib/invalidate'
import type { ImportReport } from '@/lib/types'
import { cn } from '@/lib/utils'
import { IMPORT_ERROR_KEYS } from './importErrors'

const SAMPLES = [
  { path: '/tmp/skillport-samples/my-skill', labelKey: 'import.sampleValid' },
  { path: '/tmp/skillport-samples/broken', labelKey: 'import.sampleBroken' },
  {
    path: '/tmp/skillport-samples/no-frontmatter',
    labelKey: 'import.sampleNoFrontmatter',
  },
  {
    path: '/tmp/skillport-samples/missing-skill-md',
    labelKey: 'import.sampleNoSkillFile',
  },
] as const

interface ImportDialogProps {
  onOpenChange: (open: boolean) => void
}

export function ImportDialog({ onOpenChange }: ImportDialogProps) {
  const { t } = useI18n()
  const queryClient = useQueryClient()
  const [value, setValue] = useState('')
  const [report, setReport] = useState<ImportReport | null>(null)
  const showSamples = import.meta.env.DEV && !isTauri()

  const paths = value
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)

  const run = useMutation({
    mutationFn: (items: string[]) => api.importSkills(items),
    onSuccess: (outcome) => {
      setReport(outcome)
      invalidateLibrary(queryClient)
      if (outcome.imported > 0) {
        toast(
          outcome.skipped > 0
            ? t('toast.importPartial', {
                imported: outcome.imported,
                skipped: outcome.skipped,
              })
            : t('toast.imported', { count: outcome.imported }),
          { tone: outcome.skipped > 0 ? 'warn' : 'ok' },
        )
      } else {
        toast(t('import.reportSummary', {
          imported: outcome.imported,
          skipped: outcome.skipped,
        }), { tone: 'warn' })
      }
    },
  })

  const pick = useMutation({
    mutationFn: () => api.pickImportPaths(),
    onSuccess: (picked) => {
      if (picked.length === 0) return
      setValue((current) => {
        const existing = new Set(
          current
            .split(/\r?\n/)
            .map((line) => line.trim())
            .filter(Boolean),
        )
        const merged = [...existing]
        for (const path of picked) {
          if (!existing.has(path)) merged.push(path)
        }
        return merged.join('\n')
      })
    },
    onError: (error) => {
      toast(t('toast.operationFailed', { message: toErrorMessage(error) }), {
        tone: 'err',
      })
    },
  })

  const addSample = (path: string) => {
    setValue((current) => {
      const lines = current
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter(Boolean)
      if (lines.includes(path)) return current
      return [...lines, path].join('\n')
    })
  }

  const reset = () => {
    setReport(null)
    setValue('')
  }

  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open && !run.isPending) onOpenChange(false)
      }}
      title={t('import.title')}
      description={t('import.subtitle')}
      className="w-[min(640px,94vw)]"
      footer={
        report ? (
          <>
            <Button variant="secondary" onClick={reset}>
              {t('import.another')}
            </Button>
            <Button variant="primary" onClick={() => onOpenChange(false)}>
              {t('common.done')}
            </Button>
          </>
        ) : (
          <>
            <Button
              variant="secondary"
              onClick={() => onOpenChange(false)}
              disabled={run.isPending}
            >
              {t('common.cancel')}
            </Button>
            <Button
              variant="primary"
              onClick={() => run.mutate(paths)}
              disabled={paths.length === 0 || run.isPending}
            >
              {run.isPending
                ? t('import.running')
                : t('import.run', { count: paths.length })}
            </Button>
          </>
        )
      }
    >
      {report ? (
        <div className="space-y-3">
          <div
            className={cn(
              'rounded-md border px-3 py-2 text-[13px]',
              report.skipped > 0
                ? 'border-warn-soft bg-warn-soft text-warn'
                : 'border-ok-soft bg-ok-soft text-ok',
            )}
          >
            {t('import.reportSummary', {
              imported: report.imported,
              skipped: report.skipped,
            })}
          </div>

          <div className="rounded-md border border-line">
            {report.items.map((item, index) => (
              <div
                key={`${item.path}-${index}`}
                className="flex items-start gap-2.5 border-b border-line px-3 py-2 last:border-b-0"
              >
                <span
                  className={cn(
                    'mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-full',
                    item.ok
                      ? 'bg-ok-soft text-ok'
                      : 'bg-danger-soft text-danger',
                  )}
                >
                  {item.ok ? (
                    <Check size={10} aria-hidden />
                  ) : (
                    <X size={10} aria-hidden />
                  )}
                </span>
                <div className="min-w-0 flex-1">
                  <div className="font-mono text-[11px] break-all text-ink-2">
                    {item.path}
                  </div>
                  <div className="mt-0.5 text-[11px] text-muted">
                    {item.ok
                      ? `${t('import.imported')} · ${item.slug ?? ''}`
                      : item.errorCode
                        ? t(IMPORT_ERROR_KEYS[item.errorCode])
                        : t('import.skipped')}
                  </div>
                  {!item.ok && item.detail ? (
                    <div className="mt-0.5 text-[10.5px] leading-relaxed text-faint">
                      {item.detail}
                    </div>
                  ) : null}
                </div>
              </div>
            ))}
          </div>

          <p className="text-xs leading-relaxed text-faint">
            {t('import.reportHint')}
          </p>
        </div>
      ) : (
        <div className="space-y-3.5">
          <Field
            label={t('import.pathsLabel')}
            htmlFor="import-paths"
            hint={!isTauri() ? t('import.browserHint') : undefined}
          >
            <Textarea
              id="import-paths"
              value={value}
              onChange={(event) => setValue(event.target.value)}
              placeholder={t('import.pathsPlaceholder')}
              spellCheck={false}
              className="h-32"
            />
          </Field>

          <div className="flex flex-wrap items-center gap-2">
            <Button
              variant="secondary"
              size="sm"
              onClick={() => pick.mutate()}
              disabled={!isTauri() || pick.isPending}
            >
              <FolderOpen size={13} aria-hidden />
              {t('import.choose')}
            </Button>
            {showSamples ? (
              <>
                <span className="font-mono text-[10.5px] text-faint">
                  {t('import.samples')}
                </span>
                {SAMPLES.map((sample) => (
                  <button
                    key={sample.path}
                    type="button"
                    onClick={() => addSample(sample.path)}
                    className="rounded border border-line px-1.5 py-0.5 text-[11px] text-muted transition-colors hover:border-line-2 hover:bg-panel-2 hover:text-ink"
                  >
                    {t(sample.labelKey)}
                  </button>
                ))}
              </>
            ) : null}
          </div>

          {paths.length === 0 ? (
            <p className="text-xs text-faint">{t('import.empty')}</p>
          ) : null}

          {run.isError ? (
            <p role="alert" className="text-xs text-danger">
              {toErrorMessage(run.error)}
            </p>
          ) : null}
        </div>
      )}
    </Dialog>
  )
}
