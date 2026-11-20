import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'

import type { IgnoredType } from '@/bindings/IgnoredType'

import { IgnoredTypes } from './IgnoredTypes'

const sketch: IgnoredType = { ext: 'sketch', count: 3, lastSeen: '2026-11-20T10:00:00Z', example: 'logo.sketch' }

describe('IgnoredTypes', () => {
  it('lists types and offers to create a rule', async () => {
    const user = userEvent.setup()
    const onCreateRule = vi.fn()
    const onDismiss = vi.fn()
    render(<IgnoredTypes types={[sketch]} onCreateRule={onCreateRule} onDismiss={onDismiss} onClear={() => {}} />)
    expect(screen.getByText('.sketch')).toBeInTheDocument()
    expect(screen.getByText(/3 files/)).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Create rule' }))
    expect(onCreateRule).toHaveBeenCalledWith(sketch)
    await user.click(screen.getByRole('button', { name: 'Remove from list' }))
    expect(onDismiss).toHaveBeenCalledWith(sketch)
  })

  it('renders nothing without types', () => {
    const { container } = render(
      <IgnoredTypes types={[]} onCreateRule={() => {}} onDismiss={() => {}} onClear={() => {}} />,
    )
    expect(container).toBeEmptyDOMElement()
  })
})
