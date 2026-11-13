import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'

import type { ConflictRequest } from '@/bindings/ConflictRequest'
import { Dialog, DialogContent } from '@/components/ui/dialog'

import { ConflictView } from './ConflictDialog'

const request: ConflictRequest = {
  id: 1,
  index: 0,
  conflict: { incoming: '/in/a.pdf', existing: '/docs/a.pdf', isUndo: false },
  existing: { path: '/docs/a.pdf', name: 'a.pdf', size: 2048, modified: '2026-01-01T10:00:00Z' },
  incoming: { path: '/in/a.pdf', name: 'a.pdf', size: 2048, modified: '2026-02-01T10:00:00Z' },
  identical: true,
}

function renderView(onAnswer = vi.fn()) {
  render(
    <Dialog open>
      <DialogContent>
        <ConflictView request={request} onAnswer={onAnswer} />
      </DialogContent>
    </Dialog>,
  )
  return onAnswer
}

describe('ConflictView', () => {
  it('shows both files and the identical hint', () => {
    renderView()
    expect(screen.getByText('A file with this name already exists')).toBeInTheDocument()
    expect(screen.getByText('Existing file')).toBeInTheDocument()
    expect(screen.getByText('File being moved')).toBeInTheDocument()
    expect(screen.getAllByText('2 KB')).toHaveLength(2)
    expect(screen.getByText('The two files are identical')).toBeInTheDocument()
  })

  it('answers with apply to all', async () => {
    const user = userEvent.setup()
    const onAnswer = renderView()
    await user.click(screen.getByLabelText('Apply to all remaining conflicts'))
    await user.click(screen.getByRole('button', { name: 'Replace' }))
    expect(onAnswer).toHaveBeenCalledWith('overwrite', true)
  })

  it('answers keep both and skip', async () => {
    const user = userEvent.setup()
    const onAnswer = renderView()
    await user.click(screen.getByRole('button', { name: 'Keep both' }))
    await user.click(screen.getByRole('button', { name: 'Skip' }))
    expect(onAnswer.mock.calls).toEqual([
      ['keep-both', false],
      ['skip', false],
    ])
  })
})
