import { describe, expect, it } from 'vitest'

import { fetchNewerRelease } from './release'

const fakeFetch = (status: number, body: unknown) =>
  (async () => new Response(JSON.stringify(body), { status })) as unknown as typeof fetch

describe('fetchNewerRelease', () => {
  it('returns a newer release', async () => {
    const r = await fetchNewerRelease(
      '2.0.0',
      fakeFetch(200, { tag_name: 'v2.1.0', html_url: 'https://example.test/r' }),
    )
    expect(r).toEqual({ version: '2.1.0', url: 'https://example.test/r' })
  })

  it('ignores same or older versions, drafts and errors', async () => {
    expect(await fetchNewerRelease('2.1.0', fakeFetch(200, { tag_name: 'v2.1.0', html_url: 'x' }))).toBeNull()
    expect(await fetchNewerRelease('2.1.0', fakeFetch(200, { tag_name: 'v2.0.0', html_url: 'x' }))).toBeNull()
    expect(
      await fetchNewerRelease('2.0.0', fakeFetch(200, { tag_name: 'v3.0.0', html_url: 'x', draft: true })),
    ).toBeNull()
    expect(await fetchNewerRelease('2.0.0', fakeFetch(404, {}))).toBeNull()
  })
})
