import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'

import { newRule } from './newRule'
import { RuleDialog } from './RuleDialog'

describe('RuleDialog', () => {
  it('edits and saves a rule', async () => {
    const user = userEvent.setup()
    const onSave = vi.fn()
    render(<RuleDialog rule={newRule()} isNew onClose={() => {}} onSave={onSave} />)

    expect(screen.getByRole('heading', { name: 'New rule' })).toBeInTheDocument()
    await user.type(screen.getByLabelText('Name'), '  Invoices ')
    await user.click(screen.getByLabelText('Regex'))
    await user.type(screen.getByLabelText('Pattern'), '^inv')
    await user.click(screen.getByLabelText('Case sensitive'))
    await user.type(screen.getByLabelText('Destination folder'), '/tmp/invoices')
    await user.click(screen.getByRole('button', { name: 'Save' }))

    expect(onSave).toHaveBeenCalledOnce()
    expect(onSave.mock.calls[0][0]).toMatchObject({
      name: 'Invoices',
      kind: 'regex',
      pattern: '^inv',
      caseSensitive: true,
      destination: '/tmp/invoices',
      enabled: true,
    })
  })

  it('shows help for the selected pattern type', async () => {
    const user = userEvent.setup()
    render(<RuleDialog rule={newRule()} isNew={false} onClose={() => {}} onSave={() => {}} />)
    expect(screen.getByRole('heading', { name: 'Edit rule' })).toBeInTheDocument()
    expect(screen.getByText(/\*\.\{jpg,png\}/)).toBeInTheDocument()
    await user.click(screen.getByLabelText('Regex'))
    expect(screen.getByText(/Regular expression/)).toBeInTheDocument()
  })

  it('is closed without a rule', () => {
    render(<RuleDialog rule={null} isNew onClose={() => {}} onSave={() => {}} />)
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  })
})
