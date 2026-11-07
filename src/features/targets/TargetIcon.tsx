import { Box } from 'lucide-react'
import { useMemo } from 'react'
import { useTargetIcons } from '@/lib/queries'
import { cn } from '@/lib/utils'

/**
 * Icons live in the data directory (`<data_dir>/icons/<slug>.svg`), so a new
 * harness can be added without repackaging. A mark is drawn as a mask, which
 * keeps dark-on-dark logos readable: the root `fill` decides the colour — a hex
 * keeps the brand colour, `currentColor` follows the theme.
 */
const BRAND_FILL = /\sfill="(#[0-9a-fA-F]{3,8})"/

interface TargetIconProps {
  slug: string
  /** Tile size classes; defaults to a 28px square used by tables. */
  className?: string
}

export function TargetIcon({ slug, className }: TargetIconProps) {
  const icons = useTargetIcons()
  const svg = icons.data?.find((icon) => icon.slug === slug)?.svg

  const icon = useMemo(() => {
    if (!svg) return null
    const root = svg.slice(0, svg.indexOf('>') + 1)
    return {
      mask: `url("data:image/svg+xml,${encodeURIComponent(svg)}")`,
      color: BRAND_FILL.exec(root)?.[1],
    }
  }, [svg])

  return (
    <span
      aria-hidden
      className={cn(
        'flex size-7 shrink-0 items-center justify-center rounded-md bg-raise text-muted',
        className,
      )}
    >
      {icon ? (
        <span
          className="size-4"
          style={{
            backgroundColor: icon.color ?? 'currentColor',
            maskImage: icon.mask,
            WebkitMaskImage: icon.mask,
            maskSize: 'contain',
            WebkitMaskSize: 'contain',
            maskRepeat: 'no-repeat',
            WebkitMaskRepeat: 'no-repeat',
            maskPosition: 'center',
            WebkitMaskPosition: 'center',
          }}
        />
      ) : (
        <Box size={14} aria-hidden />
      )}
    </span>
  )
}
