import {
  closestCenter,
  DndContext,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from '@dnd-kit/core'
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable'
import { open, save } from '@tauri-apps/plugin-dialog'
import { Download, MoreHorizontal, Plus, RotateCcw, Upload } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

import type { IgnoredType } from '@/bindings/IgnoredType'
import type { Rule } from '@/bindings/Rule'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { PageHeader } from '@/components/PageHeader'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { api, errorMessage, inTauri } from '@/lib/api'

import { IgnoredTypes } from './IgnoredTypes'
import { newRule } from './newRule'
import { RuleDialog } from './RuleDialog'
import { RuleRow } from './RuleRow'

export function RulesPage() {
  const { t } = useTranslation()
  const [rules, setRules] = useState<Rule[] | null>(() => (inTauri() ? null : []))
  const [editing, setEditing] = useState<{ rule: Rule; isNew: boolean } | null>(null)
  const [deleting, setDeleting] = useState<Rule | null>(null)
  const [confirmDefaults, setConfirmDefaults] = useState(false)
  const [ignored, setIgnored] = useState<IgnoredType[]>([])

  const loadIgnored = useCallback(() => {
    if (!inTauri()) return
    api
      .listIgnoredTypes()
      .then(setIgnored)
      .catch((e) => toast.error(errorMessage(e)))
  }, [])

  useEffect(() => {
    if (!inTauri()) return
    api
      .getRules()
      .then((set) => setRules(set.rules))
      .catch((e) => toast.error(errorMessage(e)))
    loadIgnored()
  }, [loadIgnored])

  /** Applies a change locally and persists it; reverts on failure. */
  const commit = useCallback(
    async (next: Rule[]) => {
      const previous = rules
      setRules(next)
      if (!inTauri()) return
      try {
        await api.saveRules({ version: 1, rules: next })
        // A new or changed rule may now cover some ignored types.
        loadIgnored()
      } catch (e) {
        setRules(previous)
        toast.error(errorMessage(e))
      }
    },
    [rules, loadIgnored],
  )

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  )

  const onDragEnd = ({ active, over }: DragEndEvent) => {
    if (!rules || !over || active.id === over.id) return
    const from = rules.findIndex((r) => r.id === active.id)
    const to = rules.findIndex((r) => r.id === over.id)
    void commit(arrayMove(rules, from, to))
  }

  const saveRule = (rule: Rule) => {
    if (!rules) return
    const exists = rules.some((r) => r.id === rule.id)
    void commit(exists ? rules.map((r) => (r.id === rule.id ? rule : r)) : [...rules, rule])
    setEditing(null)
  }

  const importRules = async () => {
    const path = await open({ filters: [{ name: t('rules.jsonFilter'), extensions: ['json'] }] })
    if (typeof path !== 'string') return
    try {
      const set = await api.importRules(path)
      setRules(set.rules)
      toast.success(t('rules.imported', { count: set.rules.length }))
    } catch (e) {
      toast.error(errorMessage(e))
    }
  }

  const exportRules = async () => {
    const path = await save({
      defaultPath: 'ordo-rules.json',
      filters: [{ name: t('rules.jsonFilter'), extensions: ['json'] }],
    })
    if (!path) return
    try {
      await api.exportRules(path)
      toast.success(t('rules.exported'))
    } catch (e) {
      toast.error(errorMessage(e))
    }
  }

  const restoreDefaults = async () => {
    setConfirmDefaults(false)
    try {
      await commit(await api.defaultRules())
    } catch (e) {
      toast.error(errorMessage(e))
    }
  }

  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title={t('rules.title')}
        description={t('rules.subtitle')}
        actions={
          <>
            <Button onClick={() => setEditing({ rule: newRule(), isNew: true })}>
              <Plus data-icon="inline-start" />
              {t('rules.add')}
            </Button>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="outline" size="icon" aria-label="More">
                  <MoreHorizontal />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem onSelect={importRules}>
                  <Upload />
                  {t('rules.import')}
                </DropdownMenuItem>
                <DropdownMenuItem onSelect={exportRules}>
                  <Download />
                  {t('rules.export')}
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem onSelect={() => setConfirmDefaults(true)}>
                  <RotateCcw />
                  {t('rules.restoreDefaults')}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </>
        }
      />

      {rules === null ? (
        <p className="text-muted-foreground text-sm">{t('common.loading')}</p>
      ) : rules.length === 0 ? (
        <p className="text-muted-foreground rounded-lg border border-dashed p-8 text-center text-sm">
          {t('rules.empty')}
        </p>
      ) : (
        <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={onDragEnd}>
          <SortableContext items={rules.map((r) => r.id)} strategy={verticalListSortingStrategy}>
            <ul className="flex flex-col gap-2">
              {rules.map((rule, i) => (
                <RuleRow
                  key={rule.id}
                  rule={rule}
                  position={i + 1}
                  onToggle={(enabled) =>
                    void commit(rules.map((r) => (r.id === rule.id ? { ...r, enabled } : r)))
                  }
                  onEdit={() => setEditing({ rule, isNew: false })}
                  onDelete={() => setDeleting(rule)}
                />
              ))}
            </ul>
          </SortableContext>
        </DndContext>
      )}

      <IgnoredTypes
        types={ignored}
        onCreateRule={(type) =>
          setEditing({
            rule: {
              ...newRule(),
              name: t('rules.ignoredRuleName', { ext: type.ext.toUpperCase() }),
              pattern: `*.${type.ext}`,
            },
            isNew: true,
          })
        }
        onDismiss={(type) => {
          setIgnored((list) => list.filter((x) => x.ext !== type.ext))
          api.dismissIgnoredType(type.ext).catch((e) => toast.error(errorMessage(e)))
        }}
        onClear={() => {
          setIgnored([])
          api.clearIgnoredTypes().catch((e) => toast.error(errorMessage(e)))
        }}
      />

      <RuleDialog
        rule={editing?.rule ?? null}
        isNew={editing?.isNew ?? false}
        onClose={() => setEditing(null)}
        onSave={saveRule}
      />

      <ConfirmDialog
        open={deleting !== null}
        title={t('rules.deleteTitle', { name: deleting?.name ?? '' })}
        description={t('rules.deleteDescription')}
        confirmLabel={t('common.delete')}
        destructive
        onCancel={() => setDeleting(null)}
        onConfirm={() => {
          if (rules && deleting) void commit(rules.filter((r) => r.id !== deleting.id))
          setDeleting(null)
        }}
      />

      <ConfirmDialog
        open={confirmDefaults}
        title={t('rules.restoreDefaultsTitle')}
        description={t('rules.restoreDefaultsDescription')}
        confirmLabel={t('rules.restoreDefaults')}
        onCancel={() => setConfirmDefaults(false)}
        onConfirm={restoreDefaults}
      />
    </div>
  )
}
