import { CheckCircle2, Circle } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

import type { CommandStatus } from '@/bindings/CommandStatus'
import type { IntegrationStatus } from '@/bindings/IntegrationStatus'
import { Button } from '@/components/ui/button'
import { api, errorMessage, inTauri } from '@/lib/api'

function platformHelpKey(status: IntegrationStatus): string {
  const id = status.items[0]?.id
  if (id === 'macos-quick-action') return 'settings.contextMenuHelpMac'
  if (id === 'windows-explorer') return 'settings.contextMenuHelpWindows'
  return 'settings.contextMenuHelp'
}

function StateLabel({ installed, label }: { installed: boolean; label: string }) {
  return (
    <span className="text-muted-foreground flex items-center gap-1.5 text-xs">
      {installed ? (
        <CheckCircle2 className="size-3.5 text-emerald-600 dark:text-emerald-400" />
      ) : (
        <Circle className="size-3.5" />
      )}
      {label}
    </span>
  )
}

export function IntegrationSection() {
  const { t } = useTranslation()
  const [menu, setMenu] = useState<IntegrationStatus | null>(null)
  const [cmd, setCmd] = useState<CommandStatus | null>(null)
  const [busy, setBusy] = useState(false)

  useEffect(() => {
    if (!inTauri()) return
    api.integrationStatus().then(setMenu).catch((e) => toast.error(errorMessage(e)))
    api.commandStatus().then(setCmd).catch((e) => toast.error(errorMessage(e)))
  }, [])

  const run = async <T,>(action: () => Promise<T>, apply: (v: T) => void, message: string) => {
    setBusy(true)
    try {
      apply(await action())
      toast.success(message)
    } catch (e) {
      toast.error(errorMessage(e))
    } finally {
      setBusy(false)
    }
  }

  if (!menu || !cmd) return null

  const available = menu.items.filter((i) => i.available)
  const anyInstalled = menu.items.some((i) => i.installed)
  const allInstalled = available.length > 0 && available.every((i) => i.installed)

  return (
    <div className="flex flex-col gap-5">
      <div className="flex flex-col gap-2">
        <div className="flex items-start justify-between gap-4">
          <div>
            <p className="text-sm">{t('settings.contextMenu')}</p>
            <p className="text-muted-foreground text-xs">{t(platformHelpKey(menu))}</p>
          </div>
          <div className="flex shrink-0 gap-2">
            {anyInstalled && (
              <Button
                variant="outline"
                size="sm"
                disabled={busy}
                onClick={() =>
                  run(api.integrationUninstall, setMenu, t('settings.integrationRemoved'))
                }
              >
                {t('settings.uninstall')}
              </Button>
            )}
            {!allInstalled && available.length > 0 && (
              <Button
                size="sm"
                disabled={busy}
                onClick={() => run(api.integrationInstall, setMenu, t('settings.integrationInstalled'))}
              >
                {t('settings.install')}
              </Button>
            )}
          </div>
        </div>
        {available.length === 0 ? (
          <p className="text-muted-foreground text-xs">{t('settings.noFileManager')}</p>
        ) : (
          <ul className="flex flex-col gap-1">
            {menu.items.map((item) => (
              <li key={item.id} className="flex items-center justify-between gap-4 text-xs">
                <span title={item.location}>{item.name}</span>
                <StateLabel
                  installed={item.installed}
                  label={
                    !item.available && !item.installed
                      ? t('settings.notAvailable')
                      : item.installed
                        ? t('settings.installed')
                        : t('settings.notInstalled')
                  }
                />
              </li>
            ))}
          </ul>
        )}
      </div>

      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0">
          <p className="text-sm">
            {t('settings.command')} <code className="bg-muted rounded px-1 font-mono text-xs">ordo</code>
          </p>
          <p className="text-muted-foreground text-xs">
            {cmd.managed
              ? t('settings.commandManaged')
              : cmd.installed && !cmd.onPath && cmd.location
                ? t('settings.commandNotOnPath', { location: cmd.location })
                : t('settings.commandHelp')}
          </p>
          <div className="mt-1">
            <StateLabel
              installed={cmd.installed}
              label={cmd.installed ? t('settings.installed') : t('settings.notInstalled')}
            />
          </div>
        </div>
        {!cmd.managed && (
          <Button
            variant={cmd.installed ? 'outline' : 'default'}
            size="sm"
            disabled={busy}
            onClick={() =>
              cmd.installed
                ? run(api.commandUninstall, setCmd, t('settings.commandRemoved'))
                : run(api.commandInstall, setCmd, t('settings.commandInstalled'))
            }
          >
            {cmd.installed ? t('settings.uninstall') : t('settings.install')}
          </Button>
        )}
      </div>
    </div>
  )
}
