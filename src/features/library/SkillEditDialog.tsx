import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useMemo, useState } from 'react'
import { Button } from '@/components/ui/button'
import { Dialog } from '@/components/ui/dialog'
import { Field } from '@/components/ui/field'
import { Input, Textarea } from '@/components/ui/input'
import { toast } from '@/components/ui/toast-store'
import { useI18n, type MessageKey } from '@/i18n'
import { api } from '@/lib/api'
import { toErrorMessage } from '@/lib/errors'
import { parseSkillMd } from '@/lib/frontmatter'
import { invalidateLibrary } from '@/lib/invalidate'
import { useSkills } from '@/lib/queries'
import { queryKeys } from '@/lib/queryKeys'
import type { ImportErrorCode, SkillDetail, SkillPatch } from '@/lib/types'
import { IMPORT_ERROR_KEYS } from './importErrors'

const META_ERRORS: Partial<Record<ImportErrorCode, MessageKey>> = {
  InvalidFrontmatter: 'edit.validationMissing',
  MissingName: 'edit.validationName',
  MissingDescription: 'edit.validationDescription',
}

interface SkillEditDialogProps {
  skill: SkillDetail
  onOpenChange: (open: boolean) => void
}

export function SkillEditDialog({
  skill,
  onOpenChange,
}: SkillEditDialogProps) {
  const { t } = useI18n()
  const queryClient = useQueryClient()
  const skills = useSkills().data
  const [value, setValue] = useState(skill.skillMd)
  // 分类是 SkillPort 自己的字段，存在库里；不从 SKILL.md 读写。
  const [category, setCategory] = useState(skill.category ?? '')
  const parsed = useMemo(() => parseSkillMd(value), [value])
  const knownCategories = useMemo(() => {
    const names = new Set<string>()
    for (const item of skills ?? []) {
      if (item.category) names.add(item.category)
    }
    return [...names].sort((a, b) => a.localeCompare(b))
  }, [skills])

  const mdChanged = value !== skill.skillMd
  const categoryChanged = category.trim() !== (skill.category ?? '')
  const patch = useMemo<SkillPatch>(() => {
    const next: SkillPatch = {}
    if (mdChanged) next.skillMd = value
    if (categoryChanged) next.category = category.trim()
    return next
  }, [mdChanged, value, categoryChanged, category])

  const save = useMutation({
    mutationFn: () => api.updateSkill(skill.id, patch),
    onSuccess: () => {
      invalidateLibrary(queryClient)
      void queryClient.invalidateQueries({
        queryKey: queryKeys.skill(skill.id),
      })
      toast(t('edit.saved', { slug: skill.slug }))
      onOpenChange(false)
    },
    onError: (error) => {
      toast(t('toast.operationFailed', { message: toErrorMessage(error) }), {
        tone: 'err',
      })
    },
  })

  const validationMessage = parsed.ok
    ? null
    : t(
        META_ERRORS[parsed.code] ??
          IMPORT_ERROR_KEYS[parsed.code] ??
          'edit.validationMissing',
      )

  const canSave = mdChanged
    ? parsed.ok && value.trim().length > 0
    : categoryChanged

  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open && !save.isPending) onOpenChange(false)
      }}
      title={t('edit.title')}
      className="w-[min(720px,94vw)]"
      bodyClassName="space-y-3"
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
            variant="primary"
            onClick={() => save.mutate()}
            disabled={!canSave || save.isPending}
          >
            {save.isPending ? t('common.loading') : t('common.save')}
          </Button>
        </>
      }
    >
      <Field label={t('edit.category')} htmlFor="skill-category">
        <Input
          id="skill-category"
          list="skill-category-options"
          value={category}
          onChange={(event) => setCategory(event.target.value)}
          placeholder={t('edit.categoryPlaceholder')}
          disabled={!parsed.ok}
          autoComplete="off"
        />
        <datalist id="skill-category-options">
          {knownCategories.map((name) => (
            <option key={name} value={name} />
          ))}
        </datalist>
      </Field>

      <Field
        label={t('edit.label')}
        htmlFor="skill-md-editor"
        error={validationMessage}
      >
        <Textarea
          id="skill-md-editor"
          value={value}
          onChange={(event) => setValue(event.target.value)}
          spellCheck={false}
          className="h-[46vh] min-h-56 text-[11.5px]"
        />
      </Field>
      {save.isError ? (
        <p role="alert" className="text-xs text-danger">
          {toErrorMessage(save.error)}
        </p>
      ) : null}
    </Dialog>
  )
}
