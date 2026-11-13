import type { Rule } from '@/bindings/Rule'

export function newRule(): Rule {
  return {
    id: crypto.randomUUID(),
    name: '',
    enabled: true,
    kind: 'glob',
    pattern: '',
    caseSensitive: false,
    destination: '',
  }
}
