import { FolderPlus, RefreshCw, Search, Upload, X } from 'lucide-react'
import { useEffect, useMemo, type KeyboardEvent, type ReactNode } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { EmptyState } from '@/components/ui/empty-state'
import { ErrorPanel } from '@/components/ui/error-panel'
import { Input } from '@/components/ui/input'
import { LoadingBlock } from '@/components/ui/spinner'
import { useI18n } from '@/i18n'
import { toErrorMessage } from '@/lib/errors'
import { filterSkills } from '@/lib/filter'
import { useSettings, useSkills, useTargets } from '@/lib/queries'
import type { SkillSummary, Target } from '@/lib/types'
import { useUiStore } from '@/lib/uiStore'
import { SkillFilters } from './SkillFilters'
import { SkillTable } from './SkillTable'

const EMPTY_SKILLS: SkillSummary[] = []
const EMPTY_TARGETS: Target[] = []

export function LibraryPage() {
  const { t } = useI18n()
  const navigate = useNavigate()
  const { slug } = useParams<{ slug?: string }>()

  const skillsQuery = useSkills()
  const targetsQuery = useTargets()
  const libraryPath = useSettings().data?.libraryPath ?? null
  const skills = skillsQuery.data ?? EMPTY_SKILLS
  const targets = targetsQuery.data ?? EMPTY_TARGETS

  const query = useUiStore((state) => state.query)
  const setQuery = useUiStore((state) => state.setQuery)
  const chip = useUiStore((state) => state.chip)
  const category = useUiStore((state) => state.category)
  const targetFilter = useUiStore((state) => state.targetFilter)
  const clearFilters = useUiStore((state) => state.clearFilters)
  const openDialog = useUiStore((state) => state.openDialog)
  const setDetailDrawerOpen = useUiStore(
    (state) => state.setDetailDrawerOpen,
  )

  const filtered = useMemo(
    () =>
      filterSkills(skills, targets, {
        query,
        chip,
        category,
        targetId: targetFilter,
      }),
    [skills, targets, query, chip, category, targetFilter],
  )

  const selected = slug
    ? (skills.find((skill) => skill.slug === slug) ?? null)
    : null

  useEffect(() => {
    if (slug) setDetailDrawerOpen(true)
  }, [slug, setDetailDrawerOpen])

  const select = (skill: SkillSummary) => {
    setDetailDrawerOpen(true)
    navigate(`/skills/${encodeURIComponent(skill.slug)}`)
  }

  const onSearchKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter' && selected) {
      event.preventDefault()
      setDetailDrawerOpen(true)
    }
  }

  const loading = skillsQuery.isLoading || targetsQuery.isLoading
  const error = skillsQuery.error ?? targetsQuery.error

  let body: ReactNode
  if (loading) {
    body = <LoadingBlock label={t('common.loading')} />
  } else if (error) {
    body = (
      <ErrorPanel
        title={t('error.skills')}
        message={toErrorMessage(error)}
        retryLabel={t('common.retry')}
        onRetry={() => {
          void skillsQuery.refetch()
          void targetsQuery.refetch()
        }}
      />
    )
  } else if (skills.length === 0) {
    body = (
      <EmptyState
        icon={<Upload size={17} aria-hidden />}
        title={t('library.emptyTitle')}
        body={t('library.emptyBody')}
        actions={
          <>
            <Button
              variant="primary"
              size="sm"
              onClick={() => openDialog({ kind: 'import' })}
            >
              {t('library.emptyActionImport')}
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => openDialog({ kind: 'target-create' })}
            >
              <FolderPlus size={13} aria-hidden />
              {t('library.emptyActionTargets')}
            </Button>
          </>
        }
      />
    )
  } else if (filtered.length === 0) {
    body = (
      <EmptyState
        icon={<Search size={17} aria-hidden />}
        title={t('library.noMatchTitle')}
        body={t('library.noMatchBody')}
        actions={
          <Button variant="secondary" size="sm" onClick={clearFilters}>
            {t('library.clearFilters')}
          </Button>
        }
      />
    )
  } else {
    body = (
      <SkillTable
        skills={filtered}
        targets={targets}
        selectedSlug={slug ?? null}
        libraryPath={libraryPath}
        onSelect={select}
      />
    )
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <header className="shrink-0 px-6 pt-5 pb-3">
        <div className="flex items-start gap-3">
          <div className="min-w-0">
            <h1 className="text-[21px] leading-none font-semibold tracking-tight text-ink">
              {t('library.title')}
            </h1>
          </div>
          <div className="ml-auto flex shrink-0 items-center gap-2">
            <Button
              variant="secondary"
              size="sm"
              onClick={() => openDialog({ kind: 'import' })}
            >
              <Upload size={13} aria-hidden />
              {t('library.import')}
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={() => openDialog({ kind: 'sync' })}
              disabled={skills.length === 0}
            >
              <RefreshCw size={13} aria-hidden />
              {t('library.syncAll')}
            </Button>
          </div>
        </div>

        <div className="relative mt-4">
          <Search
            size={13}
            aria-hidden
            className="pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2 text-faint"
          />
          <Input
            type="search"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={onSearchKeyDown}
            placeholder={t('library.searchPlaceholder')}
            aria-label={t('library.searchLabel')}
            className="h-8 pr-8 pl-8 text-xs [&::-webkit-search-cancel-button]:hidden"
          />
          {query ? (
            <button
              type="button"
              onClick={() => setQuery('')}
              aria-label={t('common.close')}
              className="absolute top-1/2 right-1.5 -translate-y-1/2 rounded p-1 text-faint transition-colors hover:text-ink"
            >
              <X size={12} aria-hidden />
            </button>
          ) : null}
        </div>
      </header>

      {skills.length > 0 ? (
        <SkillFilters skills={skills} targets={targets} />
      ) : null}

      <div className="flex min-h-0 flex-1 flex-col">{body}</div>
    </div>
  )
}
