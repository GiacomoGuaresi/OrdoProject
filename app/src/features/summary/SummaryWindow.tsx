import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { CheckCircle2, CircleAlert, Info, Undo2 } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'

import type { BatchDetail } from '@/bindings/BatchDetail'
import { Button } from '@/components/ui/button'
import { ConflictDialog } from '@/features/conflicts/ConflictDialog'
import { summarizeCounts, undoReportMessages, type UndoMessage } from '@/features/history/summary'
import { api, errorMessage, inTauri } from '@/lib/api'

/** Seconds before the summary closes by itself. */
const AUTO_CLOSE_SECONDS = 10

function closeWindow() {
  if (inTauri()) void getCurrentWebviewWindow().close()
}

type Phase =
  | { kind: 'loading' }
  | { kind: 'ready'; batch: BatchDetail | null }
  | { kind: 'undoing'; batch: BatchDetail }
  | { kind: 'done'; messages: UndoMessage[] }
  | { kind: 'error'; message: string }

/** Toast-like window shown after a sort started from the file manager. */
export function SummaryWindow({ batchId }: { batchId: number | null }) {
  const { t } = useTranslation()
  const [phase, setPhase] = useState<Phase>(() =>
    batchId === null || !inTauri() ? { kind: 'ready', batch: null } : { kind: 'loading' },
  )
  const [remaining, setRemaining] = useState(AUTO_CLOSE_SECONDS)
  const hovered = useRef(false)

  useEffect(() => {
    if (batchId === null || !inTauri()) return
    api
      .getBatch(batchId)
      .then((batch) => setPhase({ kind: 'ready', batch }))
      .catch((e) => setPhase({ kind: 'error', message: errorMessage(e) }))
  }, [batchId])

  // Countdown, paused while hovered, loading or undoing.
  const counting = phase.kind !== 'undoing' && phase.kind !== 'loading'
  useEffect(() => {
    if (!counting) return
    const timer = setInterval(() => {
      if (!hovered.current) setRemaining((r) => r - 1)
    }, 1000)
    return () => clearInterval(timer)
  }, [counting])

  useEffect(() => {
    if (remaining <= 0) closeWindow()
  }, [remaining])

  const undo = async (batch: BatchDetail) => {
    setPhase({ kind: 'undoing', batch })
    try {
      const report = await api.undoBatch(batch.id)
      setPhase({ kind: 'done', messages: undoReportMessages(t, report) })
    } catch (e) {
      setPhase({ kind: 'error', message: errorMessage(e) })
    }
    setRemaining(5)
  }

  const details = async () => {
    await api.openMain(batchId)
    closeWindow()
  }

  let icon = <Info className="text-muted-foreground size-5 shrink-0" />
  let title = t('summary.nothing')
  let subtitle: string | null = null
  let batch: BatchDetail | null = null
  switch (phase.kind) {
    case 'loading':
      title = t('common.loading')
      break
    case 'ready':
    case 'undoing': {
      batch = phase.batch
      if (!batch) break
      const moved = batch.counts.done + batch.counts.undone + batch.counts.notUndoable
      title = moved > 0 ? t('summary.sorted', { count: moved }) : t('summary.nothing')
      subtitle = summarizeCounts(t, batch.counts)
      icon =
        batch.counts.error > 0 ? (
          <CircleAlert className="text-destructive size-5 shrink-0" />
        ) : (
          <CheckCircle2 className="size-5 shrink-0 text-emerald-600 dark:text-emerald-400" />
        )
      break
    }
    case 'done':
      title = t('summary.undone')
      subtitle = phase.messages.map((m) => m.text).join(' · ')
      icon = <Undo2 className="text-muted-foreground size-5 shrink-0" />
      break
    case 'error':
      title = t('summary.failed')
      subtitle = phase.message
      icon = <CircleAlert className="text-destructive size-5 shrink-0" />
      break
  }
  const canUndo = batch !== null && batch.counts.done > 0

  return (
    <div
      className="bg-background text-foreground flex h-screen flex-col justify-between gap-3 p-4 select-none"
      onMouseEnter={() => (hovered.current = true)}
      onMouseLeave={() => (hovered.current = false)}
    >
      <div className="flex items-start gap-3">
        {icon}
        <div className="min-w-0 flex-1">
          <p className="font-medium">{title}</p>
          {subtitle && <p className="text-muted-foreground line-clamp-3 text-sm">{subtitle}</p>}
        </div>
      </div>
      <div className="flex items-center justify-between gap-2">
        <span className="text-muted-foreground text-xs tabular-nums" aria-hidden>
          {counting && remaining > 0 ? `${remaining}s` : ''}
        </span>
        <div className="flex gap-2">
          {batchId !== null && (
            <Button variant="outline" size="sm" onClick={details}>
              {t('summary.details')}
            </Button>
          )}
          {canUndo && batch && (
            <Button size="sm" disabled={phase.kind === 'undoing'} onClick={() => undo(batch)}>
              <Undo2 data-icon="inline-start" />
              {t('summary.undo')}
            </Button>
          )}
        </div>
      </div>
      <ConflictDialog />
    </div>
  )
}
