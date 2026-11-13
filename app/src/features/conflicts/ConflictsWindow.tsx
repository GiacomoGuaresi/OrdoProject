import { ConflictView } from './ConflictDialog'
import { useConflictQueue } from './useConflictQueue'

/** Window that only shows conflict questions (sort from the file manager). */
export function ConflictsWindow() {
  const { current, answer } = useConflictQueue()
  return (
    <div className="bg-background text-foreground h-screen overflow-y-auto p-5">
      {current && <ConflictView key={current.id} request={current} onAnswer={answer} />}
    </div>
  )
}
