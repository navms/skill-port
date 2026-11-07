import { useMemo } from 'react'
import { useUiStore } from '@/lib/uiStore'
import type { Language } from '@/lib/types'
import { en } from './en'
import { zhCN, type MessageKey } from './zh-CN'

const dictionaries: Record<Language, Record<MessageKey, string>> = {
  'zh-CN': zhCN,
  en,
}

export type TranslateVars = Record<string, string | number>

export function translate(
  language: Language,
  key: MessageKey,
  vars?: TranslateVars,
): string {
  const template = dictionaries[language][key] ?? zhCN[key]
  if (!vars) return template
  return template.replace(/\{\{(\w+)\}\}/g, (_, name: string) =>
    String(vars[name] ?? ''),
  )
}

export interface I18n {
  language: Language
  t: (key: MessageKey, vars?: TranslateVars) => string
}

export function useI18n(): I18n {
  const language = useUiStore((state) => state.language)
  return useMemo(
    () => ({
      language,
      t: (key: MessageKey, vars?: TranslateVars) =>
        translate(language, key, vars),
    }),
    [language],
  )
}

export type { MessageKey }
