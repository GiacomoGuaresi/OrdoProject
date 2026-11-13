import { describe, expect, it } from 'vitest'

import en from './en'
import itMessages from './it'

function keys(obj: object, prefix = ''): string[] {
  return Object.entries(obj).flatMap(([k, v]) =>
    typeof v === 'object' && v !== null ? keys(v, `${prefix}${k}.`) : [`${prefix}${k}`],
  )
}

function placeholders(obj: object, path: string): string[] {
  const value = path.split('.').reduce<unknown>((o, k) => (o as Record<string, unknown>)[k], obj)
  return [...String(value).matchAll(/{{(\w+)}}/g)].map((m) => m[1]).sort()
}

describe('locales', () => {
  it('have the same keys', () => {
    expect(keys(itMessages).sort()).toEqual(keys(en).sort())
  })

  it('use the same placeholders', () => {
    for (const k of keys(en)) expect(placeholders(itMessages, k), k).toEqual(placeholders(en, k))
  })
})
