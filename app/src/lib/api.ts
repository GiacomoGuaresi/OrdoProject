import { invoke } from '@tauri-apps/api/core'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

import type { BatchDetail } from '@/bindings/BatchDetail'
import type { CommandStatus } from '@/bindings/CommandStatus'
import type { IntegrationStatus } from '@/bindings/IntegrationStatus'
import type { BatchSummary } from '@/bindings/BatchSummary'
import type { ConflictRequest } from '@/bindings/ConflictRequest'
import type { Decision } from '@/bindings/Decision'
import type { Lang } from '@/bindings/Lang'
import type { Rule } from '@/bindings/Rule'
import type { RuleSet } from '@/bindings/RuleSet'
import type { Settings } from '@/bindings/Settings'
import type { UndoOutcome } from '@/bindings/UndoOutcome'
import type { UndoReport } from '@/bindings/UndoReport'

/** Typed wrappers around the Tauri commands of `app/src-tauri/src/commands.rs`. */
export const api = {
  systemLang: () => invoke<Lang>('system_lang'),
  getSettings: () => invoke<Settings>('get_settings'),
  saveSettings: (settings: Settings) => invoke<void>('save_settings', { settings }),

  getRules: () => invoke<RuleSet>('get_rules'),
  validateRule: (rule: Rule) => invoke<string | null>('validate_rule', { rule }),
  saveRules: (rules: RuleSet) => invoke<void>('save_rules', { rules }),
  importRules: (path: string) => invoke<RuleSet>('import_rules', { path }),
  exportRules: (path: string) => invoke<void>('export_rules', { path }),
  defaultRules: () => invoke<Rule[]>('default_rules'),

  listBatches: (limit: number) => invoke<BatchSummary[]>('list_batches', { limit }),
  getBatch: (id: number) => invoke<BatchDetail>('get_batch', { id }),
  clearHistory: () => invoke<void>('clear_history'),
  undoBatch: (id: number) => invoke<UndoReport>('undo_batch', { id }),
  undoMove: (id: number) => invoke<UndoOutcome>('undo_move', { id }),

  resolveConflict: (id: number, decision: Decision) =>
    invoke<boolean>('resolve_conflict', { id, decision }),
  revealPath: (path: string) => invoke<void>('reveal_path', { path }),
  fileThumbnail: (path: string) => invoke<string | null>('file_thumbnail', { path }),
  previewFile: (path: string) => invoke<void>('preview_file', { path }),
  pendingConflicts: () => invoke<ConflictRequest[]>('pending_conflicts'),
  openMain: (batch: number | null) => invoke<void>('open_main', { batch }),

  integrationStatus: () => invoke<IntegrationStatus>('integration_status'),
  integrationInstall: () => invoke<IntegrationStatus>('integration_install'),
  integrationUninstall: () => invoke<IntegrationStatus>('integration_uninstall'),
  commandStatus: () => invoke<CommandStatus>('command_status'),
  commandInstall: () => invoke<CommandStatus>('command_install'),
  commandUninstall: () => invoke<CommandStatus>('command_uninstall'),
}

export function onOpenBatch(handler: (batch: number) => void): Promise<UnlistenFn> {
  return getCurrentWebviewWindow().listen<number>('open-batch', (e) => handler(e.payload))
}

export function onConflict(handler: (req: ConflictRequest) => void): Promise<UnlistenFn> {
  // Scoped to this window: conflicts are addressed to a specific window.
  return getCurrentWebviewWindow().listen<ConflictRequest>('conflict', (e) => handler(e.payload))
}

/** Whether the page runs inside Tauri (false in unit tests / plain browser). */
export const inTauri = () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

/** Turns an error thrown by `invoke` into a message. */
export function errorMessage(e: unknown): string {
  if (typeof e === 'string') return e
  if (e instanceof Error) return e.message
  return String(e)
}
