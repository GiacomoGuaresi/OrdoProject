import { getVersion } from '@tauri-apps/api/app'
import { openUrl } from '@tauri-apps/plugin-opener'
import { Sparkles, X } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { inTauri } from '@/lib/api'
import { useSettings } from '@/lib/settings'

import { fetchNewerRelease, type Release } from './release'

const DISMISSED_KEY = 'ordo.update.dismissed'

function readDismissed(): string | null {
  try {
    return localStorage.getItem(DISMISSED_KEY)
  } catch {
    return null
  }
}

export function UpdateBanner() {
  const { t } = useTranslation()
  const { settings } = useSettings()
  const [release, setRelease] = useState<Release | null>(null)

  useEffect(() => {
    if (!settings.checkUpdates || !inTauri()) return
    let cancelled = false
    getVersion()
      .then((v) => fetchNewerRelease(v))
      .then((r) => {
        if (!cancelled && r && r.version !== readDismissed()) setRelease(r)
      })
      .catch(() => {
        /* offline or rate limited: try again next launch */
      })
    return () => {
      cancelled = true
    }
  }, [settings.checkUpdates])

  if (!release) return null

  const dismiss = () => {
    try {
      localStorage.setItem(DISMISSED_KEY, release.version)
    } catch {
      /* ignore */
    }
    setRelease(null)
  }

  return (
    <div
      role="status"
      className="bg-primary/5 flex items-center gap-3 border-b px-4 py-2 text-sm"
    >
      <Sparkles className="text-primary size-4 shrink-0" />
      <span className="flex-1">{t('update.available', { version: release.version })}</span>
      <Button size="sm" onClick={() => void openUrl(release.url)}>
        {t('update.download')}
      </Button>
      <Button variant="ghost" size="icon-sm" onClick={dismiss} aria-label={t('update.dismiss')}>
        <X />
      </Button>
    </div>
  )
}
