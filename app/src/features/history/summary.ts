import type { TFunction } from 'i18next'

import type { StatusCounts } from '@/bindings/StatusCounts'
import type { UndoOutcome } from '@/bindings/UndoOutcome'
import type { UndoReport } from '@/bindings/UndoReport'

/** "3 moved · 1 ignored · 1 error" */
export function summarizeCounts(t: TFunction, c: StatusCounts): string {
  const parts: string[] = []
  const moved = c.done + c.undone + c.notUndoable
  parts.push(t('history.summary.moved', { count: moved }))
  if (c.undone) parts.push(t('history.summary.undone', { count: c.undone }))
  if (c.skipped) parts.push(t('history.summary.skipped', { count: c.skipped }))
  if (c.ignored) parts.push(t('history.summary.ignored', { count: c.ignored }))
  if (c.error) parts.push(t('history.summary.error', { count: c.error }))
  return parts.join(' · ')
}

export interface UndoMessage {
  kind: 'success' | 'warning' | 'error' | 'info'
  text: string
}

/** Messages to show after undoing a batch. */
export function undoReportMessages(t: TFunction, r: UndoReport): UndoMessage[] {
  const out: UndoMessage[] = []
  const nothing = r.undone + r.notUndoable + r.skipped + r.errors.length === 0
  if (nothing) return [{ kind: 'info', text: t('history.undoResult.nothing') }]
  if (r.undone) out.push({ kind: 'success', text: t('history.undoResult.undone', { count: r.undone }) })
  if (r.notUndoable)
    out.push({ kind: 'warning', text: t('history.undoResult.notUndoable', { count: r.notUndoable }) })
  if (r.skipped) out.push({ kind: 'info', text: t('history.undoResult.skipped', { count: r.skipped }) })
  if (r.errors.length)
    out.push({ kind: 'error', text: t('history.undoResult.errors', { count: r.errors.length }) })
  for (const path of r.manualTrashRestores)
    out.push({ kind: 'warning', text: t('history.undoResult.trashManual', { path }) })
  return out
}

/** Messages to show after undoing a single move. */
export function undoOutcomeMessages(t: TFunction, o: UndoOutcome): UndoMessage[] {
  switch (o.kind) {
    case 'undone': {
      const out: UndoMessage[] = [{ kind: 'success', text: t('history.undoResult.undone', { count: 1 }) }]
      if (o.trash?.kind === 'restored') out.push({ kind: 'info', text: t('history.undoResult.trashRestored') })
      if (o.trash?.kind === 'manual')
        out.push({ kind: 'warning', text: t('history.undoResult.trashManual', { path: o.trash.path }) })
      return out
    }
    case 'not_undoable':
      return [{ kind: 'warning', text: t('history.undoResult.notUndoable', { count: 1 }) }]
    case 'skipped':
      return [{ kind: 'info', text: t('history.undoResult.skipped', { count: 1 }) }]
    case 'not_applicable':
      return [{ kind: 'info', text: t('history.undoResult.nothing') }]
    case 'failed':
      return [{ kind: 'error', text: `${t(`errors.${o.error.code}`)} — ${o.error.detail}` }]
  }
}
