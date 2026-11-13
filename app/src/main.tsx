import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

import './index.css'
import '@/lib/i18n'
import App from './App'
import { SettingsProvider } from '@/lib/SettingsProvider'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <SettingsProvider>
      <App />
    </SettingsProvider>
  </StrictMode>,
)
