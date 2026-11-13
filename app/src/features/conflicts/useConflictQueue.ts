import { useEffect, useState } from 'react'

import type { ConflictRequest } from '@/bindings/ConflictRequest'
import type { Resolution } from '@/bindings/Resolution'
import { api, inTauri, onConflict } from '@/lib/api'

/** Conflicts addressed to this window, oldest first, and a way to answer them. */
export function useConflictQueue() {
  const [queue, setQueue] = useState<ConflictRequest[]>([])

  useEffect(() => {
    if (!inTauri()) return
    const add = (reqs: ConflictRequest[]) =>
      setQueue((q) => [...q, ...reqs.filter((r) => !q.some((x) => x.id === r.id))])
    const un = onConflict((req) => add([req]))
    // Requests sent before this window was ready.
    void api.pendingConflicts().then(add)
    return () => {
      void un.then((f) => f())
    }
  }, [])

  const current = queue[0]

  const answer = (resolution: Resolution, applyToAll: boolean) => {
    if (!current) return
    void api.resolveConflict(current.id, { resolution, applyToAll })
    setQueue((q) => q.slice(1))
  }

  return { current, answer }
}
