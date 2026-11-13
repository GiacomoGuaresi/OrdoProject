import { useCallback, useEffect, useMemo, useState, type ReactNode } from 'react'

import type { Lang } from '@/bindings/Lang'
import type { Settings } from '@/bindings/Settings'
import { api, inTauri } from '@/lib/api'
import i18n from '@/lib/i18n'
import { DEFAULT_SETTINGS, SettingsContext, useSystemDark } from '@/lib/settings'

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<Settings | null>(() => (inTauri() ? null : DEFAULT_SETTINGS))
  const [systemLang, setSystemLang] = useState<Lang>('en')
  const systemDark = useSystemDark()

  useEffect(() => {
    if (!inTauri()) return
    Promise.all([api.getSettings(), api.systemLang()])
      .then(([s, l]) => {
        setSystemLang(l)
        setSettings(s)
      })
      .catch(() => setSettings(DEFAULT_SETTINGS))
  }, [])

  const lang = settings?.language ?? systemLang
  useEffect(() => {
    void i18n.changeLanguage(lang)
    document.documentElement.lang = lang
  }, [lang])

  const theme = settings?.theme ?? 'system'
  const dark = theme === 'dark' || (theme === 'system' && systemDark)
  useEffect(() => {
    document.documentElement.classList.toggle('dark', dark)
    document.documentElement.style.colorScheme = dark ? 'dark' : 'light'
  }, [dark])

  const update = useCallback(
    async (patch: Partial<Settings>) => {
      if (!settings) return
      const next = { ...settings, ...patch }
      setSettings(next)
      if (inTauri()) await api.saveSettings(next)
    },
    [settings],
  )

  const value = useMemo(
    () => (settings ? { settings, systemLang, update } : null),
    [settings, systemLang, update],
  )

  if (!value) return null
  return <SettingsContext.Provider value={value}>{children}</SettingsContext.Provider>
}
