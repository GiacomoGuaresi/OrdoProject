import { describe, expect, it } from 'vitest'

import i18n from '@/lib/i18n'

import { summarizeCounts, undoOutcomeMessages, undoReportMessages } from './summary'

const t = i18n.getFixedT('en')

const counts = {
  done: 2,
  undone: 1,
  notUndoable: 0,
  skipped: 0,
  ignored: 1,
  alreadyInPlace: 0,
  error: 1,
}

describe('history summaries', () => {
  it('summarizes counts', () => {
    expect(summarizeCounts(t, counts)).toBe('3 moved · 1 undone · 1 ignored · 1 error')
  })

  it('describes undo reports', () => {
    const msgs = undoReportMessages(t, {
      batchId: 1,
      undone: 2,
      notUndoable: 1,
      skipped: 0,
      errors: [],
      manualTrashRestores: ['/x/a.pdf'],
    })
    expect(msgs.map((m) => m.kind)).toEqual(['success', 'warning', 'warning'])
    expect(msgs[2].text).toContain('/x/a.pdf')
    expect(
      undoReportMessages(t, {
        batchId: 1,
        undone: 0,
        notUndoable: 0,
        skipped: 0,
        errors: [],
        manualTrashRestores: [],
      })[0].text,
    ).toBe('Nothing to undo')
  })

  it('describes single undo outcomes', () => {
    expect(undoOutcomeMessages(t, { kind: 'undone', restoredTo: '/a', trash: null })[0].kind).toBe('success')
    expect(
      undoOutcomeMessages(t, { kind: 'failed', error: { code: 'volume_missing', detail: '/Volumes/X' } })[0].text,
    ).toBe('Destination drive not connected — /Volumes/X')
  })

  it('translates to Italian', () => {
    expect(summarizeCounts(i18n.getFixedT('it'), { ...counts, undone: 0, ignored: 0, error: 0 })).toBe(
      '2 spostati',
    )
  })
})
