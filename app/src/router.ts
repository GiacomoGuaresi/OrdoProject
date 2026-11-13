export type Route =
  | { view: 'main'; batch: number | null }
  | { view: 'conflicts' }
  | { view: 'summary'; batch: number | null }

/** Parses the window's hash: `#/conflicts`, `#/summary/12`, `#/history/12`. */
export function parseRoute(hash: string): Route {
  const parts = hash.replace(/^#\/?/, '').split('/').filter(Boolean)
  const id = parts[1] !== undefined && /^\d+$/.test(parts[1]) ? Number(parts[1]) : null
  switch (parts[0]) {
    case 'conflicts':
      return { view: 'conflicts' }
    case 'summary':
      return { view: 'summary', batch: id }
    case 'history':
      return { view: 'main', batch: id }
    default:
      return { view: 'main', batch: null }
  }
}
