/** Human-readable file size. */
export function formatSize(bytes: number, locale?: string): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = bytes
  let unit = 0
  while (size >= 1024 && unit < units.length - 1) {
    size /= 1024
    unit++
  }
  const value = unit === 0 ? String(size) : size.toLocaleString(locale, { maximumFractionDigits: 1 })
  return `${value} ${units[unit]}`
}

/** Short local date and time. */
export function formatDateTime(iso: string | null | undefined, locale?: string): string {
  if (!iso) return '—'
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return '—'
  return d.toLocaleString(locale, { dateStyle: 'medium', timeStyle: 'short' })
}

/** Last path component (works with `/` and `\`). */
export function baseName(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean)
  return parts[parts.length - 1] ?? path
}

/** Parent folder of a path. */
export function dirName(path: string): string {
  const i = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'))
  return i > 0 ? path.slice(0, i) : path
}

/**
 * Compares two versions like `2.1.0`, `v2.0.0-alpha.1`.
 * Returns a negative number if a < b, 0 if equal, positive if a > b.
 */
export function compareVersions(a: string, b: string): number {
  const parse = (v: string) => {
    const [core, pre] = v.replace(/^v/i, '').split('-', 2)
    return { nums: core.split('.').map((n) => Number.parseInt(n, 10) || 0), pre: pre ?? null }
  }
  const pa = parse(a)
  const pb = parse(b)
  for (let i = 0; i < Math.max(pa.nums.length, pb.nums.length); i++) {
    const d = (pa.nums[i] ?? 0) - (pb.nums[i] ?? 0)
    if (d !== 0) return d
  }
  if (pa.pre === pb.pre) return 0
  if (pa.pre === null) return 1
  if (pb.pre === null) return -1
  return pa.pre.localeCompare(pb.pre, undefined, { numeric: true })
}
