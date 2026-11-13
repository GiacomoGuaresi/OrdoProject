import { Check, History, ListChecks, MousePointerClick, type LucideIcon } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { api, errorMessage, inTauri } from '@/lib/api'
import { useSettings } from '@/lib/settings'

const STEPS: { icon: LucideIcon; key: 'step1' | 'step2' | 'step3' }[] = [
  { icon: ListChecks, key: 'step1' },
  { icon: MousePointerClick, key: 'step2' },
  { icon: History, key: 'step3' },
]

/** Shown once, on the first launch of the app. */
export function OnboardingDialog() {
  const { t } = useTranslation()
  const { settings, update } = useSettings()
  const [installed, setInstalled] = useState(false)
  const [busy, setBusy] = useState(false)

  const install = async () => {
    setBusy(true)
    try {
      await api.integrationInstall()
      setInstalled(true)
    } catch (e) {
      toast.error(errorMessage(e))
    } finally {
      setBusy(false)
    }
  }

  const finish = () => {
    update({ onboardingDone: true }).catch((e) => toast.error(errorMessage(e)))
  }

  return (
    <Dialog open={!settings.onboardingDone} onOpenChange={(o) => !o && finish()}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t('onboarding.title')}</DialogTitle>
          <DialogDescription>{t('onboarding.intro')}</DialogDescription>
        </DialogHeader>
        <ol className="flex flex-col gap-4">
          {STEPS.map(({ icon: Icon, key }) => (
            <li key={key} className="flex gap-3">
              <div className="bg-muted flex size-9 shrink-0 items-center justify-center rounded-full">
                <Icon className="size-4" />
              </div>
              <div>
                <p className="text-sm font-medium">{t(`onboarding.${key}Title`)}</p>
                <p className="text-muted-foreground text-sm">{t(`onboarding.${key}`)}</p>
              </div>
            </li>
          ))}
        </ol>
        <DialogFooter className="gap-2 sm:justify-between">
          <Button variant="outline" onClick={install} disabled={busy || installed || !inTauri()}>
            {installed && <Check data-icon="inline-start" />}
            {installed ? t('onboarding.installed') : t('onboarding.install')}
          </Button>
          <Button onClick={finish}>{t('onboarding.start')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
