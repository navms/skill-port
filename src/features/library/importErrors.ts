import type { MessageKey } from '@/i18n'
import type { ImportErrorCode } from '@/lib/types'

export const IMPORT_ERROR_KEYS: Record<ImportErrorCode, MessageKey> = {
  MissingSkillFile: 'import.error.MissingSkillFile',
  InvalidFrontmatter: 'import.error.InvalidFrontmatter',
  MissingName: 'import.error.MissingName',
  MissingDescription: 'import.error.MissingDescription',
  InvalidSlug: 'import.error.InvalidSlug',
  DuplicateSlug: 'import.error.DuplicateSlug',
  PathNotFound: 'import.error.PathNotFound',
  IoError: 'import.error.IoError',
}
