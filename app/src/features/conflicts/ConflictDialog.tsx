import { CopyCheck, FolderSearch } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'

import type { ConflictRequest } from '@/bindings/ConflictRequest'
import type { FileMeta } from '@/bindings/FileMeta'
import type { Resolution } from '@/bindings/Resolution'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import { api } from '@/lib/api'
import { baseName, dirName, formatDateTime, formatSize } from '@/lib/format'

import { useConflictQueue } from './useConflictQueue'

function FileCard({ title, file }: { title: string; file: FileMeta }) {
  const { t, i18n } = useTranslation()
  return (
    <div className="bg-muted/40 flex min-w-0 flex-col gap-2 rounded-lg border p-3">
      <div className="text-muted-foreground text-xs font-medium tracking-wide uppercase">{title}</div>
      <div className="truncate text-sm font-medium" title={file.path}>
        {file.name}
      </div>
      <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
        <dt className="text-muted-foreground">{t('conflict.size')}</dt>
        <dd>{formatSize(file.size, i18n.language)}</dd>
        <dt className="text-muted-foreground">{t('conflict.modified')}</dt>
        <dd>{formatDateTime(file.modified, i18n.language)}</dd>
      </dl>
      <Button
        variant="ghost"
        size="sm"
        className="self-start"
        onClick={() => void api.revealPath(file.path)}
      >
        <FolderSearch data-icon="inline-start" />
        {t('conflict.reveal')}
      </Button>
    </div>
  )
}

interface ViewProps {
  request: ConflictRequest
  onAnswer: (resolution: Resolution, applyToAll: boolean) => void
}

/** The conflict question, usable in a dialog or as a whole window. */
export function ConflictView({ request, onAnswer }: ViewProps) {
  const { t } = useTranslation()
  const [applyToAll, setApplyToAll] = useState(false)
  const { conflict, existing, incoming, identical } = request
  const folder = dirName(existing.path)

  const answer = (r: Resolution) => onAnswer(r, applyToAll)

  return (
    <div className="grid gap-4">
      <div className="grid gap-1">
        <h2 className="text-base font-semibold">
          {conflict.isUndo ? t('conflict.titleUndo') : t('conflict.title')}
        </h2>
        <p className="text-muted-foreground truncate text-sm" title={folder}>
          {t('conflict.description', { name: baseName(existing.path), folder })}
        </p>
      </div>

      <div className="grid grid-cols-2 gap-3">
        <FileCard title={t('conflict.existing')} file={existing} />
        <FileCard title={t('conflict.incoming')} file={incoming} />
      </div>

      {identical && (
        <p className="flex items-center gap-2 text-sm text-emerald-600 dark:text-emerald-400">
          <CopyCheck className="size-4" />
          {t('conflict.identical')}
        </p>
      )}

      <div className="flex items-center gap-2">
        <Checkbox
          id="apply-to-all"
          checked={applyToAll}
          onCheckedChange={(v) => setApplyToAll(v === true)}
        />
        <Label htmlFor="apply-to-all" className="font-normal">
          {t('conflict.applyToAll')}
        </Label>
      </div>

      <div className="flex items-center justify-between gap-2">
        <Button variant="outline" onClick={() => answer('skip')} title={t('conflict.skipHint')}>
          {t('conflict.skip')}
        </Button>
        <div className="flex gap-2">
          <Button
            variant="destructive"
            onClick={() => answer('overwrite')}
            title={t('conflict.overwriteHint')}
          >
            {t('conflict.overwrite')}
          </Button>
          <Button onClick={() => answer('keep-both')} title={t('conflict.keepBothHint')} autoFocus>
            {t('conflict.keepBoth')}
          </Button>
        </div>
      </div>
    </div>
  )
}

/** Modal for conflicts raised by operations started from this window. */
export function ConflictDialog() {
  const { t } = useTranslation()
  const { current, answer } = useConflictQueue()
  return (
    <Dialog open={current !== undefined} onOpenChange={(o) => !o && answer('skip', false)}>
      <DialogContent className="sm:max-w-xl" showCloseButton={false}>
        <DialogTitle className="sr-only">{t('conflict.title')}</DialogTitle>
        <DialogDescription className="sr-only">{t('conflict.title')}</DialogDescription>
        {current && <ConflictView key={current.id} request={current} onAnswer={answer} />}
      </DialogContent>
    </Dialog>
  )
}
