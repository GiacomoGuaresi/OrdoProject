import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

import './index.css'
import '@/lib/i18n'
import { Toaster } from '@/components/ui/sonner'
import { ConflictsWindow } from '@/features/conflicts/ConflictsWindow'
import { SummaryWindow } from '@/features/summary/SummaryWindow'
import { SettingsProvider } from '@/lib/SettingsProvider'

import App from './App'
import { parseRoute } from './router'

const route = parseRoute(window.location.hash)

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <SettingsProvider>
      {route.view === 'conflicts' && <ConflictsWindow />}
      {route.view === 'summary' && (
        <>
          <SummaryWindow batchId={route.batch} />
          <Toaster position="top-center" richColors />
        </>
      )}
      {route.view === 'main' && (
        <App initialPage={route.page} initialBatch={route.batch} />
      )}
    </SettingsProvider>
  </StrictMode>,
)
