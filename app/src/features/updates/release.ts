import { compareVersions } from '@/lib/format'

const LATEST_RELEASE_URL = 'https://api.github.com/repos/GiacomoGuaresi/OrdoProject/releases/latest'

export interface Release {
  version: string
  url: string
}

/** Returns the latest GitHub release if it is newer than `current`. */
export async function fetchNewerRelease(
  current: string,
  fetchImpl: typeof fetch = fetch,
): Promise<Release | null> {
  const res = await fetchImpl(LATEST_RELEASE_URL, { headers: { Accept: 'application/vnd.github+json' } })
  if (!res.ok) return null
  const data = (await res.json()) as { tag_name?: string; html_url?: string; draft?: boolean }
  if (!data.tag_name || !data.html_url || data.draft) return null
  const version = data.tag_name.replace(/^v/i, '')
  return compareVersions(version, current) > 0 ? { version, url: data.html_url } : null
}
