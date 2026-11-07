import type { ReactNode } from 'react'

const KEY_LINE = /^(\s*)([A-Za-z0-9_-]+):(.*)$/

export function SkillMdPreview({ source }: { source: string }) {
  const lines = source.replace(/\r\n/g, '\n').split('\n')
  const hasFrontmatter = lines[0]?.trim() === '---'

  let closeIndex = -1
  if (hasFrontmatter) {
    for (let index = 1; index < lines.length; index += 1) {
      if (lines[index]?.trim() === '---') {
        closeIndex = index
        break
      }
    }
  }

  return (
    <pre className="mt-2 min-h-[96px] flex-1 overflow-auto rounded-md border border-line bg-code-bg px-3 py-2.5 font-mono text-[11.5px] leading-[1.75] whitespace-pre-wrap break-words text-code-ink">
      <code>
        {lines.map((line, index) => {
          const inFrontmatter =
            hasFrontmatter &&
            closeIndex > 0 &&
            index > 0 &&
            index < closeIndex
          const isDelimiter =
            hasFrontmatter && (index === 0 || index === closeIndex)
          const key = inFrontmatter ? line.match(KEY_LINE) : null

          let content: ReactNode = line || ' '
          if (isDelimiter) {
            content = <span className="text-syn-c">---</span>
          } else if (key) {
            content = (
              <>
                <span className="text-syn-k">
                  {key[1]}
                  {key[2]}:
                </span>
                <span className="text-syn-v">{key[3]}</span>
              </>
            )
          }

          return (
            <span key={index} className="block">
              {content}
            </span>
          )
        })}
      </code>
    </pre>
  )
}
