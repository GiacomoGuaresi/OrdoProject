import { describe, expect, it } from 'vitest'

import { parseRoute } from './router'

describe('parseRoute', () => {
  it('parses window routes', () => {
    expect(parseRoute('')).toEqual({ view: 'main', page: 'rules', batch: null })
    expect(parseRoute('#/history/12')).toEqual({ view: 'main', page: 'history', batch: 12 })
    expect(parseRoute('#/settings')).toEqual({ view: 'main', page: 'settings', batch: null })
    expect(parseRoute('#/conflicts')).toEqual({ view: 'conflicts' })
    expect(parseRoute('#/summary/3')).toEqual({ view: 'summary', batch: 3 })
    expect(parseRoute('#/summary')).toEqual({ view: 'summary', batch: null })
    expect(parseRoute('#/summary/x')).toEqual({ view: 'summary', batch: null })
  })
})
