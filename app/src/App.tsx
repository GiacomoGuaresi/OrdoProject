import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'

function App() {
  const [coreVersion, setCoreVersion] = useState<string | null>(null)

  useEffect(() => {
    invoke<string>('core_version')
      .then(setCoreVersion)
      .catch(() => setCoreVersion(null))
  }, [])

  return (
    <main className="app">
      <h1>Ordo</h1>
      <p>{coreVersion ? `core v${coreVersion}` : 'Running outside Tauri'}</p>
    </main>
  )
}

export default App
