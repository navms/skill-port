import { Button } from '@/components/ui/button'
import { StateDot } from '@/components/ui/status'
import { Switch } from '@/components/ui/switch'
import { useI18n } from '@/i18n'
import {
  TONE_TEXT,
  installCopy,
  installFor,
  installTone,
  stateCopy,
} from '@/lib/status'
import type { Install, Language, Target } from '@/lib/types'
import { cn } from '@/lib/utils'

interface DistributionSectionProps {
  installs: readonly Install[]
  targets: readonly Target[]
  language: Language
  pendingTargetId: string | null
  onToggle: (target: Target, enabled: boolean) => void
  onResolve: (target: Target) => void
  onUnignore: (target: Target) => void
  onManageTargets: () => void
}

export function DistributionSection({
  installs,
  targets,
  language,
  pendingTargetId,
  onToggle,
  onResolve,
  onUnignore,
  onManageTargets,
}: DistributionSectionProps) {
  const { t } = useI18n()
  const enabledTargets = targets.filter((target) => target.enabled)
  const installsForEnabled = enabledTargets
    .map((target) => installFor(installs, target.id))
    .filter((install): install is Install => Boolean(install))

  const conflicts = enabledTargets.filter((target) => {
    const install = installFor(installs, target.id)
    return install?.status === 'conflict' && !install.ignored
  })
  const ignored = enabledTargets.filter((target) => {
    const install = installFor(installs, target.id)
    return install?.status === 'conflict' && install.ignored
  })

  return (
    <section className="mt-5">
      <div className="flex items-center justify-between border-b border-line pb-1.5">
        <h3 className="font-mono text-[10px] font-medium tracking-[0.13em] text-faint uppercase">
          {t('detail.distribution')}
        </h3>
        <span className="font-mono text-[10px] text-faint">
          {t('detail.configured', {
            linked: installsForEnabled.length,
            total: enabledTargets.length,
          })}
        </span>
      </div>

      {enabledTargets.length === 0 ? (
        <p className="py-3 text-xs leading-relaxed text-muted">
          {t('detail.noInstalls')}
        </p>
      ) : (
        enabledTargets.map((target) => {
          const install = installFor(installs, target.id)
          const mode = install?.mode ?? target.linkMode
          const stateText = install
            ? installCopy(install, language)
            : stateCopy('none', mode, language)
          const tone = install ? installTone(install) : 'muted'
          const isConflict = install?.status === 'conflict'
          const isPending = pendingTargetId === target.id

          return (
            <div
              key={target.id}
              className="flex items-center gap-2.5 border-b border-line py-2.5 last:border-b-0"
            >
              <StateDot state={install?.status ?? 'none'} />
              <div className="min-w-0 flex-1">
                <b className="block truncate text-[12.5px] font-medium text-ink-2">
                  {target.name}
                </b>
                <small className="mt-0.5 block truncate text-[11px] text-faint">
                  {mode === 'copy' ? t('common.copy') : t('common.symlink')}
                </small>
              </div>
              <span
                className={cn(
                  'w-14 shrink-0 text-right text-[11px]',
                  TONE_TEXT[tone],
                )}
              >
                {stateText}
              </span>
              <Switch
                label={t('targets.toggleEnable', { name: target.name })}
                tone={isConflict ? 'conflict' : 'default'}
                checked={Boolean(install)}
                disabled={isConflict || isPending}
                onCheckedChange={(checked) => onToggle(target, checked)}
              />
            </div>
          )
        })
      )}

      {conflicts.length > 0 ? (
        <div className="mt-2 rounded-md border border-danger-soft bg-danger-soft px-2.5 py-2 text-xs leading-relaxed text-muted">
          <b className="mb-0.5 block text-[11px] text-danger">
            {t('detail.conflictTitle')}
          </b>
          {t('detail.conflictBody', {
            targets: conflicts.map((target) => target.name).join('、'),
          })}
          <div className="mt-2 flex flex-wrap gap-2">
            <Button
              variant="secondary"
              size="sm"
              onClick={() => {
                const first = conflicts[0]
                if (first) onResolve(first)
              }}
            >
              {t('detail.resolveConflict')}
            </Button>
            <Button variant="ghost" size="sm" onClick={onManageTargets}>
              {t('detail.goTargets')}
            </Button>
          </div>
        </div>
      ) : null}

      {ignored.length > 0 ? (
        <div className="mt-2 rounded-md border border-line bg-panel-2 px-2.5 py-2 text-xs leading-relaxed text-muted">
          {t('detail.ignoredConflictBody', {
            targets: ignored.map((target) => target.name).join('、'),
          })}
          <div className="mt-1.5 flex flex-wrap gap-1.5">
            {ignored.map((target) => (
              <button
                key={target.id}
                type="button"
                onClick={() => onUnignore(target)}
                className="rounded border border-line-2 px-1.5 py-0.5 text-[11px] text-muted transition-colors hover:bg-raise hover:text-ink"
              >
                {target.name} · {t('detail.unignore')}
              </button>
            ))}
          </div>
        </div>
      ) : null}
    </section>
  )
}
