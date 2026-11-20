import { open } from '@tauri-apps/plugin-dialog'
import { FolderPlus, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

import { Button } from '@/components/ui/button'
import { errorMessage, inTauri } from '@/lib/api'
import { useSettings } from '@/lib/settings'

/** Folders sorted when the "Ordo" launcher is opened. */
export function QuickSortFolders() {
  const { t } = useTranslation()
  const { settings, update } = useSettings()
  const folders = settings.quickSortFolders

  const save = (next: string[]) =>
    update({ quickSortFolders: next }).catch((e) => toast.error(errorMessage(e)))

  const add = async () => {
    const dir = await open({ directory: true })
    if (typeof dir === 'string' && !folders.includes(dir)) void save([...folders, dir])
  }

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-start justify-between gap-4">
        <div>
          <p className="text-sm">{t('settings.quickSort')}</p>
          <p className="text-muted-foreground text-xs">{t('settings.quickSortHelp')}</p>
        </div>
        <Button variant="outline" size="sm" onClick={add} disabled={!inTauri()}>
          <FolderPlus data-icon="inline-start" />
          {t('settings.addFolder')}
        </Button>
      </div>
      <ul className="flex flex-col gap-1">
        {folders.length === 0 ? (
          <li className="text-muted-foreground text-xs">{t('settings.quickSortDefault')}</li>
        ) : (
          folders.map((f) => (
            <li key={f} className="flex items-center justify-between gap-2 text-xs">
              <span className="truncate" title={f}>
                {f}
              </span>
              <Button
                variant="ghost"
                size="icon-xs"
                aria-label={t('settings.removeFolder')}
                onClick={() => void save(folders.filter((x) => x !== f))}
              >
                <X />
              </Button>
            </li>
          ))
        )}
      </ul>
    </div>
  )
}
