import { useMutation, useQueryClient } from '@tanstack/react-query'
import { FolderOpen } from 'lucide-react'
import { useState, type FormEvent } from 'react'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Dialog } from '@/components/ui/dialog'
import { Field } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Segmented } from '@/components/ui/segmented'
import { toast } from '@/components/ui/toast-store'
import { useI18n } from '@/i18n'
import { api, isTauri } from '@/lib/api'
import { toErrorMessage } from '@/lib/errors'
import { invalidateLibrary, invalidateTargets } from '@/lib/invalidate'
import { useSettings, useTargetSuggestions } from '@/lib/queries'
import type {
  LinkMode,
  SuggestedTarget,
  Target,
  TargetInput,
} from '@/lib/types'
import { SuggestionList } from './SuggestionList'
import {
  ABSOLUTE_PATH,
  SLUG_PATTERN,
  isDuplicateDir,
  isInsideLibrary,
  normalizeDir,
} from './targetPathRules'

interface TargetFormDialogProps {
  target: Target | null
  targets: readonly Target[]
  onOpenChange: (open: boolean) => void
}

export function TargetFormDialog({
  target,
  targets,
  onOpenChange,
}: TargetFormDialogProps) {
  const { t } = useI18n()
  const queryClient = useQueryClient()
  const settingsQuery = useSettings()
  const suggestionsQuery = useTargetSuggestions(true)
  const suggestions = suggestionsQuery.data ?? []

  const [name, setName] = useState(target?.name ?? '')
  const [slug, setSlug] = useState(target?.slug ?? '')
  const [skillsDir, setSkillsDir] = useState(target?.skillsDir ?? '')
  const [linkMode, setLinkMode] = useState<LinkMode>(
    target?.linkMode ?? 'symlink',
  )
  const [enabled, setEnabled] = useState(target?.enabled ?? true)
  const [touched, setTouched] = useState({
    name: false,
    slug: false,
    dir: false,
  })

  const touch = (field: keyof typeof touched) =>
    setTouched((current) => ({ ...current, [field]: true }))

  const nameError = name.trim() ? null : t('targets.validationName')

  const trimmedSlug = slug.trim()
  const slugError =
    trimmedSlug.length >= 2 &&
    trimmedSlug.length <= 63 &&
    SLUG_PATTERN.test(trimmedSlug)
      ? null
      : t('targets.validationSlug')

  const dir = normalizeDir(skillsDir)
  const libraryPath = normalizeDir(settingsQuery.data?.libraryPath ?? '')
  const duplicate = isDuplicateDir(dir, targets, target?.id)
  const insideLibrary = isInsideLibrary(dir, libraryPath)

  let dirError: string | null = null
  if (!dir) {
    dirError = t('targets.validationDir')
  } else if (!ABSOLUTE_PATH.test(dir)) {
    dirError = t('targets.validationDir')
  } else if (insideLibrary) {
    dirError = t('targets.validationDirInsideLibrary')
  } else if (duplicate) {
    dirError = t('targets.validationDirDuplicate')
  }

  const save = useMutation({
    mutationFn: (input: TargetInput) =>
      target
        ? api.updateTarget(target.id, input)
        : api.createTarget(input),
    onSuccess: (saved) => {
      invalidateTargets(queryClient)
      invalidateLibrary(queryClient)
      toast(
        target
          ? t('toast.targetUpdated', { name: saved.name })
          : t('toast.targetCreated', { name: saved.name }),
      )
      onOpenChange(false)
    },
    onError: (error) => {
      toast(t('toast.operationFailed', { message: toErrorMessage(error) }), {
        tone: 'err',
      })
    },
  })

  const browse = useMutation({
    mutationFn: () => api.pickDirectory(dir || undefined),
    onSuccess: (picked) => {
      if (picked) setSkillsDir(picked)
    },
    onError: (error) => {
      toast(t('toast.operationFailed', { message: toErrorMessage(error) }), {
        tone: 'err',
      })
    },
  })

  const pickSuggestion = (suggestion: SuggestedTarget) => {
    setName(suggestion.name)
    setSlug(suggestion.slug)
    setSkillsDir(suggestion.skillsDir)
    setTouched({ name: true, slug: true, dir: true })
  }

  const submit = (event: FormEvent) => {
    event.preventDefault()
    setTouched({ name: true, slug: true, dir: true })
    if (nameError || slugError || dirError) return
    save.mutate({
      name: name.trim(),
      slug: trimmedSlug,
      skillsDir: dir,
      linkMode,
      enabled,
    })
  }

  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open && !save.isPending) onOpenChange(false)
      }}
      title={target ? t('targets.editTitle') : t('targets.addTitle')}
      description={
        target ? t('targets.editSubtitle') : t('targets.addSubtitle')
      }
      className="w-[min(580px,94vw)]"
      footer={
        <>
          <Button
            variant="secondary"
            onClick={() => onOpenChange(false)}
            disabled={save.isPending}
          >
            {t('common.cancel')}
          </Button>
          <Button
            type="submit"
            form="target-form"
            variant="primary"
            disabled={save.isPending}
          >
            {save.isPending ? t('common.loading') : t('targets.save')}
          </Button>
        </>
      }
    >
      <form id="target-form" onSubmit={submit} className="space-y-3.5">
        {!target ? (
          <SuggestionList
            suggestions={suggestions}
            onPick={pickSuggestion}
          />
        ) : null}

        <div className="grid grid-cols-2 gap-3">
          <Field
            label={t('targets.fieldName')}
            htmlFor="target-name"
            error={touched.name ? nameError : null}
          >
            <Input
              id="target-name"
              value={name}
              onChange={(event) => setName(event.target.value)}
              onBlur={() => touch('name')}
              autoFocus
              placeholder="Claude Code"
            />
          </Field>

          <Field
            label={t('targets.fieldSlug')}
            htmlFor="target-slug"
            error={touched.slug ? slugError : null}
          >
            <Input
              id="target-slug"
              value={slug}
              onChange={(event) =>
                setSlug(event.target.value.toLowerCase().replace(/\s+/g, '-'))
              }
              onBlur={() => touch('slug')}
              placeholder="claude-code"
              className="font-mono"
              spellCheck={false}
            />
          </Field>
        </div>

        <Field
          label={t('targets.fieldDir')}
          htmlFor="target-dir"
          error={touched.dir ? dirError : null}
          hint={!isTauri() ? t('common.browserPicker') : undefined}
        >
          <div className="flex gap-2">
            <Input
              id="target-dir"
              value={skillsDir}
              onChange={(event) => setSkillsDir(event.target.value)}
              onBlur={() => touch('dir')}
              placeholder="~/.claude/skills"
              className="font-mono text-[12px]"
              spellCheck={false}
            />
            <Button
              type="button"
              variant="secondary"
              size="sm"
              onClick={() => browse.mutate()}
              disabled={!isTauri() || browse.isPending}
              className="shrink-0"
            >
              <FolderOpen size={13} aria-hidden />
              {t('targets.browse')}
            </Button>
          </div>
        </Field>

        <Field
          label={t('targets.fieldMode')}
          hint={
            linkMode === 'copy'
              ? t('targets.modeHintCopy')
              : t('targets.modeHintSymlink')
          }
        >
          <Segmented<LinkMode>
            value={linkMode}
            ariaLabel={t('targets.fieldMode')}
            onChange={setLinkMode}
            options={[
              { value: 'symlink', label: t('common.symlink') },
              { value: 'copy', label: t('common.copy') },
            ]}
          />
        </Field>

        <Checkbox
          checked={enabled}
          onChange={(event) => setEnabled(event.target.checked)}
          label={t('targets.fieldEnabled')}
        />

        {save.isError ? (
          <p role="alert" className="text-xs text-danger">
            {toErrorMessage(save.error)}
          </p>
        ) : null}
      </form>
    </Dialog>
  )
}
