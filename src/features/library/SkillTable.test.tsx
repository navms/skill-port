import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import {
  installWith,
  makeSkill,
  makeTarget,
} from '@/test/fixtures'
import { SkillTable } from './SkillTable'

const claude = makeTarget()

const alpha = makeSkill({
  id: 'skl_alpha',
  slug: 'alpha',
  description: 'Has an external conflict',
  installs: [installWith(claude.id, 'conflict')],
})

const beta = makeSkill({
  id: 'skl_beta',
  slug: 'beta',
  description: 'All linked',
  installs: [installWith(claude.id, 'ok')],
})

describe('SkillTable', () => {
  it('shows a labelled conflict count instead of colour only', () => {
    render(
      <SkillTable
        skills={[alpha]}
        targets={[claude]}
        selectedSlug={null}
        onSelect={() => {}}
      />,
    )

    expect(screen.getByText('alpha')).toBeInTheDocument()
    expect(screen.getByText('Has an external conflict')).toBeInTheDocument()
    expect(screen.getByText('1 个冲突')).toBeInTheDocument()
    expect(screen.getByText('1 / 1')).toBeInTheDocument()
  })

  it('selects a row on click and marks it as current', () => {
    const onSelect = vi.fn()
    const { rerender } = render(
      <SkillTable
        skills={[alpha, beta]}
        targets={[claude]}
        selectedSlug={null}
        onSelect={onSelect}
      />,
    )

    fireEvent.click(screen.getByRole('button', { name: /beta/ }))
    expect(onSelect).toHaveBeenCalledWith(beta)

    rerender(
      <SkillTable
        skills={[alpha, beta]}
        targets={[claude]}
        selectedSlug="beta"
        onSelect={onSelect}
      />,
    )
    expect(screen.getByRole('button', { name: /beta/ })).toHaveAttribute(
      'aria-current',
      'true',
    )
  })

  it('renders no rows for an empty library', () => {
    render(
      <SkillTable
        skills={[]}
        targets={[claude]}
        selectedSlug={null}
        onSelect={() => {}}
      />,
    )

    expect(screen.queryByRole('button')).not.toBeInTheDocument()
  })
})
