import { describe, expect, it } from 'vitest'

import { parseRoute } from './router'

describe('parseRoute', () => {
  it('parses window routes', () => {
    expect(parseRoute('')).toEqual({ view: 'main', batch: null })
    expect(parseRoute('#/history/12')).toEqual({ view: 'main', batch: 12 })
    expect(parseRoute('#/conflicts')).toEqual({ view: 'conflicts' })
    expect(parseRoute('#/summary/3')).toEqual({ view: 'summary', batch: 3 })
    expect(parseRoute('#/summary')).toEqual({ view: 'summary', batch: null })
    expect(parseRoute('#/summary/x')).toEqual({ view: 'summary', batch: null })
  })
})
