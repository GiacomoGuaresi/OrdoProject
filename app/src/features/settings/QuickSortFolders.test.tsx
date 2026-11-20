import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { SettingsProvider } from '@/lib/SettingsProvider'

import { QuickSortFolders } from './QuickSortFolders'

describe('QuickSortFolders', () => {
  it('defaults to the desktop', () => {
    render(
      <SettingsProvider>
        <QuickSortFolders />
      </SettingsProvider>,
    )
    expect(screen.getByText('Folders sorted by “Ordo”')).toBeInTheDocument()
    expect(screen.getByText('Desktop (default)')).toBeInTheDocument()
  })
})
