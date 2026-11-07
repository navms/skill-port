import { Wand2 } from 'lucide-react'
import { useI18n } from '@/i18n'
import type { SuggestedTarget } from '@/lib/types'
import { cn } from '@/lib/utils'

interface SuggestionListProps {
  suggestions: readonly SuggestedTarget[]
  onPick: (suggestion: SuggestedTarget) => void
}

export function SuggestionList({ suggestions, onPick }: SuggestionListProps) {
  const { t } = useI18n()

  return (
    <div>
      <div className="mb-1.5 flex items-center gap-1.5 font-mono text-[10px] tracking-[0.12em] text-faint uppercase">
        <Wand2 size={11} aria-hidden />
        {t('targets.suggestions')}
      </div>
      {suggestions.length > 0 ? (
        <div className="grid gap-1">
          {suggestions.map((suggestion) => (
            <button
              key={suggestion.slug}
              type="button"
              disabled={suggestion.alreadyAdded}
              onClick={() => onPick(suggestion)}
              className={cn(
                'flex items-center gap-2 rounded-md border border-line px-2.5 py-1.5 text-left transition-colors',
                suggestion.alreadyAdded
                  ? 'opacity-50'
                  : 'hover:border-line-2 hover:bg-panel-2',
              )}
            >
              <span className="min-w-0 flex-1">
                <b className="block truncate text-[12.5px] font-medium text-ink-2">
                  {suggestion.name}
                </b>
                <span className="block truncate font-mono text-[10.5px] text-faint">
                  {suggestion.skillsDir}
                </span>
              </span>
              {suggestion.alreadyAdded ? (
                <span className="shrink-0 text-[10.5px] text-faint">
                  {t('targets.alreadyAdded')}
                </span>
              ) : null}
            </button>
          ))}
        </div>
      ) : (
        <p className="text-xs leading-relaxed text-faint">
          {t('targets.noSuggestions')}
        </p>
      )}
    </div>
  )
}
