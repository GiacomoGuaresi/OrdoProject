import { describe, expect, it } from 'vitest'

import { baseName, compareVersions, dirName, formatSize } from './format'

describe('formatSize', () => {
  it('formats bytes and larger units', () => {
    expect(formatSize(12, 'en')).toBe('12 B')
    expect(formatSize(2048, 'en')).toBe('2 KB')
    expect(formatSize(1536, 'en')).toBe('1.5 KB')
    expect(formatSize(5 * 1024 * 1024, 'en')).toBe('5 MB')
  })
})

describe('paths', () => {
  it('splits unix and windows paths', () => {
    expect(baseName('/a/b/c.txt')).toBe('c.txt')
    expect(baseName('C:\\Users\\me\\file.pdf')).toBe('file.pdf')
    expect(dirName('/a/b/c.txt')).toBe('/a/b')
    expect(dirName('C:\\Users\\me\\file.pdf')).toBe('C:\\Users\\me')
  })
})

describe('compareVersions', () => {
  it('orders versions', () => {
    expect(compareVersions('2.0.0', '2.0.0')).toBe(0)
    expect(compareVersions('v2.1.0', '2.0.9')).toBeGreaterThan(0)
    expect(compareVersions('2.0.0', '2.0.10')).toBeLessThan(0)
    expect(compareVersions('2.0.0', '2.0.0-alpha.1')).toBeGreaterThan(0)
    expect(compareVersions('2.0.0-alpha.2', '2.0.0-alpha.10')).toBeLessThan(0)
    expect(compareVersions('2.0.0-beta.1', '2.0.0-alpha.3')).toBeGreaterThan(0)
  })
})
