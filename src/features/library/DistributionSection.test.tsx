import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { installWith, makeTarget } from '@/test/fixtures'
import { DistributionSection } from './DistributionSection'

const claude = makeTarget()

function renderSection(installs = [installWith(claude.id, 'conflict')], targets = [claude]) {
  return render(
    <DistributionSection
      installs={installs}
      targets={targets}
      language="zh-CN"
      pendingTargetId={null}
      onToggle={vi.fn()}
      onResolve={vi.fn()}
      onUnignore={vi.fn()}
      onManageTargets={vi.fn()}
    />,
  )
}

describe('DistributionSection', () => {
  it('surfaces an unresolved conflict without offering an overwrite', () => {
    renderSection()

    expect(screen.getByText('需要你决定')).toBeInTheDocument()
    expect(
      screen.getByRole('button', { name: '处理冲突…' }),
    ).toBeInTheDocument()
    expect(screen.getByText('冲突')).toBeInTheDocument()

    const toggle = screen.getByRole('switch')
    expect(toggle).toBeDisabled()
    expect(toggle).toHaveAttribute('data-state', 'checked')
  })

  it('explains when there are no enabled targets', () => {
    renderSection([], [])

    expect(
      screen.getByText('还没有分发到任何启用目标。'),
    ).toBeInTheDocument()
  })

  it('offers to unignore a conflict the user kept', () => {
    renderSection([
      installWith(claude.id, 'conflict', { ignored: true }),
    ])

    expect(
      screen.getByRole('button', { name: /Claude Code · 取消忽略/ }),
    ).toBeInTheDocument()
    expect(screen.getByText('已忽略')).toBeInTheDocument()
  })
})
