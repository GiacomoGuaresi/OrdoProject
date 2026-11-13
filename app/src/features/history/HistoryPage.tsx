import { ArrowRight, FolderSearch, RefreshCw, Undo2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

import type { BatchDetail } from '@/bindings/BatchDetail'
import type { BatchSummary } from '@/bindings/BatchSummary'
import type { Entry } from '@/bindings/Entry'
import type { MoveStatus } from '@/bindings/MoveStatus'
import { PageHeader } from '@/components/PageHeader'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { api, errorMessage, inTauri } from '@/lib/api'
import { baseName, dirName, formatDateTime } from '@/lib/format'
import { cn } from '@/lib/utils'

import { summarizeCounts, undoOutcomeMessages, undoReportMessages, type UndoMessage } from './summary'

const PAGE_SIZE = 200

const STATUS_VARIANT: Record<MoveStatus, 'default' | 'secondary' | 'outline' | 'destructive'> = {
  done: 'default',
  undone: 'secondary',
  not_undoable: 'outline',
  skipped: 'outline',
  ignored: 'outline',
  already_in_place: 'outline',
  error: 'destructive',
}

function showMessages(messages: UndoMessage[]) {
  for (const m of messages) toast[m.kind](m.text)
}

interface Props {
  /** Batch to select when the page opens (e.g. from a notification). */
  initialBatch?: number | null
}

export function HistoryPage({ initialBatch = null }: Props) {
  const { t, i18n } = useTranslation()
  const [batches, setBatches] = useState<BatchSummary[] | null>(() => (inTauri() ? null : []))
  const [selected, setSelected] = useState<number | null>(initialBatch)
  const [detail, setDetail] = useState<BatchDetail | null>(null)
  const [busy, setBusy] = useState(false)
  // Bumped to reload the list and the selected batch.
  const [reloadKey, setReloadKey] = useState(0)
  const refresh = () => setReloadKey((k) => k + 1)

  useEffect(() => {
    if (!inTauri()) return
    let cancelled = false
    api
      .listBatches(PAGE_SIZE)
      .then((list) => !cancelled && setBatches(list))
      .catch((e) => toast.error(errorMessage(e)))
    return () => {
      cancelled = true
    }
  }, [reloadKey])

  const current = selected ?? batches?.[0]?.id ?? null

  useEffect(() => {
    if (current === null || !inTauri()) return
    let cancelled = false
    api
      .getBatch(current)
      .then((d) => !cancelled && setDetail(d))
      .catch((e) => toast.error(errorMessage(e)))
    return () => {
      cancelled = true
    }
  }, [current, reloadKey])

  const shown = detail !== null && detail.id === current ? detail : null

  const undoBatch = async (id: number) => {
    setBusy(true)
    try {
      showMessages(undoReportMessages(t, await api.undoBatch(id)))
    } catch (e) {
      toast.error(errorMessage(e))
    } finally {
      setBusy(false)
      refresh()
    }
  }

  const undoMove = async (entry: Entry) => {
    setBusy(true)
    try {
      showMessages(undoOutcomeMessages(t, await api.undoMove(entry.id)))
    } catch (e) {
      toast.error(errorMessage(e))
    } finally {
      setBusy(false)
      refresh()
    }
  }

  const reveal = (path: string) => {
    api.revealPath(path).catch((e) => toast.error(errorMessage(e)))
  }

  const locale = i18n.language

  return (
    <div className="flex h-full min-h-0 flex-col gap-6">
      <PageHeader
        title={t('history.title')}
        description={t('history.subtitle')}
        actions={
          <Button variant="outline" size="icon" onClick={refresh} aria-label={t('common.refresh')}>
            <RefreshCw />
          </Button>
        }
      />

      {batches === null ? (
        <p className="text-muted-foreground text-sm">{t('common.loading')}</p>
      ) : batches.length === 0 ? (
        <p className="text-muted-foreground rounded-lg border border-dashed p-8 text-center text-sm">
          {t('history.empty')}
        </p>
      ) : (
        <div className="grid min-h-0 flex-1 grid-cols-[minmax(220px,280px)_1fr] gap-4">
          <ul className="flex min-h-0 flex-col gap-1 overflow-y-auto pr-1" aria-label={t('history.title')}>
            {batches.map((b) => (
              <li key={b.id}>
                <button
                  type="button"
                  onClick={() => setSelected(b.id)}
                  aria-current={b.id === current}
                  className={cn(
                    'hover:bg-muted w-full rounded-md px-3 py-2 text-left transition-colors',
                    b.id === current && 'bg-muted',
                  )}
                >
                  <div className="flex items-center justify-between gap-2 text-sm font-medium">
                    <span>{formatDateTime(b.createdAt, locale)}</span>
                    <span className="text-muted-foreground text-xs">#{b.id}</span>
                  </div>
                  <div className="text-muted-foreground text-xs">
                    {t(`history.origin.${b.origin}`)} · {summarizeCounts(t, b.counts)}
                  </div>
                </button>
              </li>
            ))}
          </ul>

          <section className="min-h-0 overflow-y-auto rounded-lg border">
            {shown === null ? (
              <p className="text-muted-foreground p-6 text-sm">{t('history.selectBatch')}</p>
            ) : (
              <div className="flex flex-col">
                <div className="bg-background/95 sticky top-0 flex items-center justify-between gap-4 border-b px-4 py-3 backdrop-blur">
                  <div>
                    <h2 className="font-medium">{t('history.batch', { id: shown.id })}</h2>
                    <p className="text-muted-foreground text-xs">
                      {formatDateTime(shown.createdAt, locale)} · {t(`history.origin.${shown.origin}`)} ·{' '}
                      {summarizeCounts(t, shown.counts)}
                    </p>
                  </div>
                  <Button
                    variant="outline"
                    disabled={busy || shown.counts.done === 0}
                    onClick={() => undoBatch(shown.id)}
                  >
                    <Undo2 data-icon="inline-start" />
                    {t('history.undoBatch')}
                  </Button>
                </div>
                <ul className="divide-y">
                  {shown.entries.map((e) => (
                    <EntryRow
                      key={e.id}
                      entry={e}
                      busy={busy}
                      onUndo={() => undoMove(e)}
                      onReveal={reveal}
                    />
                  ))}
                </ul>
              </div>
            )}
          </section>
        </div>
      )}
    </div>
  )
}

function EntryRow({
  entry,
  busy,
  onUndo,
  onReveal,
}: {
  entry: Entry
  busy: boolean
  onUndo: () => void
  onReveal: (path: string) => void
}) {
  const { t } = useTranslation()
  // Where the file is now.
  const current =
    entry.status === 'done'
      ? entry.destination
      : entry.status === 'undone'
        ? (entry.restoredTo ?? entry.source)
        : entry.source

  return (
    <li className="flex items-center gap-3 px-4 py-2.5">
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="truncate text-sm font-medium" title={entry.source}>
            {baseName(entry.source)}
          </span>
          <Badge variant={STATUS_VARIANT[entry.status]} className="shrink-0 text-[0.7rem]">
            {t(`history.status.${entry.status}`)}
          </Badge>
          {entry.ruleName && <span className="text-muted-foreground truncate text-xs">{entry.ruleName}</span>}
        </div>
        <div className="text-muted-foreground flex min-w-0 items-center gap-1.5 text-xs">
          <span className="truncate" title={dirName(entry.source)}>
            {dirName(entry.source)}
          </span>
          {entry.destination && entry.status !== 'ignored' && (
            <>
              <ArrowRight className="size-3 shrink-0" />
              <span className="truncate" title={entry.destination}>
                {dirName(entry.destination)}
              </span>
            </>
          )}
        </div>
        {entry.error && (
          <p className="text-destructive text-xs">
            {t(`errors.${entry.error.code}`)} — {entry.error.detail}
          </p>
        )}
      </div>
      {current && (
        <Button
          variant="ghost"
          size="icon-sm"
          onClick={() => onReveal(current)}
          aria-label={t('history.reveal')}
          title={t('history.reveal')}
        >
          <FolderSearch />
        </Button>
      )}
      {entry.status === 'done' && (
        <Button variant="ghost" size="sm" disabled={busy} onClick={onUndo}>
          <Undo2 data-icon="inline-start" />
          {t('history.undoMove')}
        </Button>
      )}
    </li>
  )
}
