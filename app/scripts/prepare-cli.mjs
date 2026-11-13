// Builds the `ordo` CLI in release mode and copies it where Tauri expects
// an external binary: src-tauri/binaries/ordo-<target-triple>[.exe].
// Run automatically by `pnpm bundle` (see src-tauri/tauri.bundle.conf.json).
import { execFileSync } from 'node:child_process'
import { copyFileSync, mkdirSync } from 'node:fs'
import path from 'node:path'

const root = path.resolve(import.meta.dirname, '..', '..')

function hostTriple() {
  const out = execFileSync('rustc', ['-vV'], { encoding: 'utf8' })
  return out.match(/^host: (.+)$/m)[1].trim()
}

const host = hostTriple()
const triple = process.env.TAURI_ENV_TARGET_TRIPLE || host
const crossArgs = triple === host ? [] : ['--target', triple]

execFileSync('cargo', ['build', '--release', '-p', 'ordo-cli', ...crossArgs], {
  cwd: root,
  stdio: 'inherit',
})

const ext = triple.includes('windows') ? '.exe' : ''
const built = path.join(root, 'target', ...(crossArgs.length ? [triple] : []), 'release', `ordo${ext}`)
const destDir = path.join(root, 'app', 'src-tauri', 'binaries')
mkdirSync(destDir, { recursive: true })
const dest = path.join(destDir, `ordo-${triple}${ext}`)
copyFileSync(built, dest)
console.log(`ordo CLI ready: ${path.relative(root, dest)}`)
