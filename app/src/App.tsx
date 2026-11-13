import { History, ListChecks, Settings as SettingsIcon, type LucideIcon } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Toaster } from '@/components/ui/sonner'
import { TooltipProvider } from '@/components/ui/tooltip'
import { ConflictDialog } from '@/features/conflicts/ConflictDialog'
import { HistoryPage } from '@/features/history/HistoryPage'
import { OnboardingDialog } from '@/features/onboarding/OnboardingDialog'
import { RulesPage } from '@/features/rules/RulesPage'
import { SettingsPage } from '@/features/settings/SettingsPage'
import { UpdateBanner } from '@/features/updates/UpdateBanner'
import { inTauri, onOpenBatch } from '@/lib/api'
import { cn } from '@/lib/utils'

export type Page = 'rules' | 'history' | 'settings'

const NAV: { page: Page; icon: LucideIcon }[] = [
  { page: 'rules', icon: ListChecks },
  { page: 'history', icon: History },
  { page: 'settings', icon: SettingsIcon },
]

interface Props {
  initialPage?: Page
  initialBatch?: number | null
}

export default function App({ initialPage = 'rules', initialBatch = null }: Props) {
  const { t } = useTranslation()
  const [page, setPage] = useState<Page>(initialPage)
  const [batch, setBatch] = useState<number | null>(initialBatch)

  // "Details" from a sort summary while this window is already open.
  useEffect(() => {
    if (!inTauri()) return
    const un = onOpenBatch((id) => {
      setBatch(id)
      setPage('history')
    })
    return () => {
      void un.then((f) => f())
    }
  }, [])

  return (
    <TooltipProvider>
      <div className="bg-background text-foreground flex h-screen flex-col">
        <UpdateBanner />
        <div className="flex min-h-0 flex-1">
          <nav className="bg-muted/30 flex w-52 shrink-0 flex-col gap-1 border-r p-3">
            <div className="mb-4 flex items-center gap-2 px-2 pt-1">
              <img src="/icon.png" alt="" className="size-7" />
              <span className="text-lg font-semibold tracking-tight">{t('app.name')}</span>
            </div>
            {NAV.map(({ page: p, icon: Icon }) => (
              <button
                key={p}
                type="button"
                onClick={() => setPage(p)}
                aria-current={page === p ? 'page' : undefined}
                className={cn(
                  'hover:bg-muted flex items-center gap-2.5 rounded-md px-2.5 py-1.5 text-sm transition-colors',
                  page === p && 'bg-muted font-medium',
                )}
              >
                <Icon className="size-4" />
                {t(`nav.${p}`)}
              </button>
            ))}
          </nav>
          <main className="min-h-0 flex-1 overflow-y-auto p-6">
            {page === 'rules' && <RulesPage />}
            {page === 'history' && <HistoryPage key={batch ?? -1} initialBatch={batch} />}
            {page === 'settings' && <SettingsPage />}
          </main>
        </div>
      </div>
      <ConflictDialog />
      <OnboardingDialog />
      <Toaster position="bottom-right" richColors closeButton />
    </TooltipProvider>
  )
}
