import { createContext, useContext, useEffect, useState } from 'react'

import type { Lang } from '@/bindings/Lang'
import type { Settings } from '@/bindings/Settings'

export const DEFAULT_SETTINGS: Settings = {
  version: 1,
  language: null,
  theme: 'system',
  retentionDays: 90,
  checkUpdates: true,
  onboardingDone: false,
  quickSortFolders: [],
  launcherOffered: false,
}

export interface SettingsContextValue {
  settings: Settings
  systemLang: Lang
  /** Persists a partial change and applies it immediately. */
  update: (patch: Partial<Settings>) => Promise<void>
}

export const SettingsContext = createContext<SettingsContextValue | null>(null)

export function prefersDark(): boolean {
  return typeof window !== 'undefined' && !!window.matchMedia?.('(prefers-color-scheme: dark)').matches
}

export function useSettings(): SettingsContextValue {
  const ctx = useContext(SettingsContext)
  if (!ctx) throw new Error('useSettings must be used inside <SettingsProvider>')
  return ctx
}

/** Whether the system currently prefers a dark color scheme (live). */
export function useSystemDark(): boolean {
  const [dark, setDark] = useState(prefersDark)
  useEffect(() => {
    if (!window.matchMedia) return
    const mq = window.matchMedia('(prefers-color-scheme: dark)')
    const onChange = () => setDark(mq.matches)
    mq.addEventListener('change', onChange)
    return () => mq.removeEventListener('change', onChange)
  }, [])
  return dark
}

/** Effective theme ('light' | 'dark'). */
export function useResolvedTheme(): 'light' | 'dark' {
  const { settings } = useSettings()
  const systemDark = useSystemDark()
  if (settings.theme === 'system') return systemDark ? 'dark' : 'light'
  return settings.theme
}
