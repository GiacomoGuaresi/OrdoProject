import { getVersion } from '@tauri-apps/api/app'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useEffect, useState, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

import type { Lang } from '@/bindings/Lang'
import type { Theme } from '@/bindings/Theme'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { PageHeader } from '@/components/PageHeader'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { api, errorMessage, inTauri } from '@/lib/api'
import { LANGUAGES } from '@/lib/i18n'
import { useSettings } from '@/lib/settings'

import { IntegrationSection } from './IntegrationSection'

export const REPO_URL = 'https://github.com/GiacomoGuaresi/OrdoProject'

const RETENTION_OPTIONS = [7, 30, 90, 180, 365]
const SYSTEM = 'system'
const FOREVER = 'forever'

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="flex flex-col gap-4">
      <h2 className="text-muted-foreground text-sm font-medium">{title}</h2>
      <div className="flex flex-col gap-4">{children}</div>
    </section>
  )
}

function Row({ label, htmlFor, children }: { label: string; htmlFor?: string; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <Label htmlFor={htmlFor} className="font-normal">
        {label}
      </Label>
      {children}
    </div>
  )
}

export function SettingsPage() {
  const { t } = useTranslation()
  const { settings, systemLang, update } = useSettings()
  const [confirmClear, setConfirmClear] = useState(false)
  const [version, setVersion] = useState<string | null>(null)

  useEffect(() => {
    if (inTauri()) void getVersion().then(setVersion)
  }, [])

  const save = (patch: Parameters<typeof update>[0]) =>
    update(patch).catch((e) => toast.error(errorMessage(e)))

  const clearHistory = async () => {
    setConfirmClear(false)
    try {
      await api.clearHistory()
      toast.success(t('settings.historyCleared'))
    } catch (e) {
      toast.error(errorMessage(e))
    }
  }

  return (
    <div className="flex max-w-xl flex-col gap-8">
      <PageHeader title={t('settings.title')} />

      <Section title={t('settings.appearance')}>
        <Row label={t('settings.theme')} htmlFor="theme">
          <Select value={settings.theme} onValueChange={(v) => save({ theme: v as Theme })}>
            <SelectTrigger id="theme" className="w-44">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="system">{t('settings.themeSystem')}</SelectItem>
              <SelectItem value="light">{t('settings.themeLight')}</SelectItem>
              <SelectItem value="dark">{t('settings.themeDark')}</SelectItem>
            </SelectContent>
          </Select>
        </Row>
        <Row label={t('settings.language')} htmlFor="language">
          <Select
            value={settings.language ?? SYSTEM}
            onValueChange={(v) => save({ language: v === SYSTEM ? null : (v as Lang) })}
          >
            <SelectTrigger id="language" className="w-44">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={SYSTEM}>
                {t('settings.languageSystem', { lang: LANGUAGES[systemLang] })}
              </SelectItem>
              {Object.entries(LANGUAGES).map(([code, name]) => (
                <SelectItem key={code} value={code}>
                  {name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Row>
      </Section>

      <Separator />

      <Section title={t('settings.integration')}>
        <IntegrationSection />
      </Section>

      <Separator />

      <Section title={t('settings.history')}>
        <Row label={t('settings.retention')} htmlFor="retention">
          <Select
            value={settings.retentionDays === null ? FOREVER : String(settings.retentionDays)}
            onValueChange={(v) => save({ retentionDays: v === FOREVER ? null : Number(v) })}
          >
            <SelectTrigger id="retention" className="w-44">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {RETENTION_OPTIONS.map((d) => (
                <SelectItem key={d} value={String(d)}>
                  {t('settings.retentionDays', { count: d })}
                </SelectItem>
              ))}
              <SelectItem value={FOREVER}>{t('settings.retentionForever')}</SelectItem>
            </SelectContent>
          </Select>
        </Row>
        <div>
          <Button variant="destructive" onClick={() => setConfirmClear(true)}>
            {t('settings.clearHistory')}
          </Button>
        </div>
      </Section>

      <Separator />

      <Section title={t('settings.updates')}>
        <Row label={t('settings.checkUpdates')} htmlFor="check-updates">
          <Switch
            id="check-updates"
            checked={settings.checkUpdates}
            onCheckedChange={(v) => save({ checkUpdates: v })}
          />
        </Row>
      </Section>

      <Separator />

      <Section title={t('settings.about')}>
        <div className="flex items-center justify-between gap-4 text-sm">
          <span>
            Ordo {version && <span className="text-muted-foreground">· {t('settings.version', { version })}</span>}
          </span>
          <Button variant="link" className="h-auto p-0" onClick={() => void openUrl(REPO_URL)}>
            {t('settings.sourceCode')}
          </Button>
        </div>
      </Section>

      <ConfirmDialog
        open={confirmClear}
        title={t('settings.clearHistoryTitle')}
        description={t('settings.clearHistoryDescription')}
        confirmLabel={t('settings.clearHistory')}
        destructive
        onCancel={() => setConfirmClear(false)}
        onConfirm={clearHistory}
      />
    </div>
  )
}
