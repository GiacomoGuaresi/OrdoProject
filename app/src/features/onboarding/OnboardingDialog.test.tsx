import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'

import { SettingsProvider } from '@/lib/SettingsProvider'

import { OnboardingDialog } from './OnboardingDialog'

describe('OnboardingDialog', () => {
  it('is shown until the user starts', async () => {
    const user = userEvent.setup()
    render(
      <SettingsProvider>
        <OnboardingDialog />
      </SettingsProvider>,
    )
    expect(screen.getByRole('heading', { name: 'Welcome to Ordo' })).toBeInTheDocument()
    expect(screen.getByText('Sort from the file manager')).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Get started' }))
    expect(screen.queryByRole('heading', { name: 'Welcome to Ordo' })).not.toBeInTheDocument()
  })
})
