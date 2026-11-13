import { open } from '@tauri-apps/plugin-dialog'
import { FolderOpen } from 'lucide-react'
import { useState, type FormEvent } from 'react'
import { useTranslation } from 'react-i18next'

import type { PatternKind } from '@/bindings/PatternKind'
import type { Rule } from '@/bindings/Rule'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { Switch } from '@/components/ui/switch'
import { api, inTauri } from '@/lib/api'

interface Props {
  /** The rule being edited, or a fresh rule; `null` closes the dialog. */
  rule: Rule | null
  isNew: boolean
  onClose: () => void
  onSave: (rule: Rule) => void
}

export function RuleDialog({ rule, isNew, onClose, onSave }: Props) {
  return (
    <Dialog open={rule !== null} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-lg">
        {rule && <RuleForm key={rule.id} initial={rule} isNew={isNew} onClose={onClose} onSave={onSave} />}
      </DialogContent>
    </Dialog>
  )
}

interface FormProps {
  initial: Rule
  isNew: boolean
  onClose: () => void
  onSave: (rule: Rule) => void
}

function RuleForm({ initial, isNew, onClose, onSave }: FormProps) {
  const { t } = useTranslation()
  const [draft, setDraft] = useState<Rule>(initial)
  const [error, setError] = useState<string | null>(null)

  const set = <K extends keyof Rule>(key: K, value: Rule[K]) => {
    setDraft((d) => ({ ...d, [key]: value }))
    setError(null)
  }

  const browse = async () => {
    const dir = await open({ directory: true, defaultPath: draft.destination || undefined })
    if (typeof dir === 'string') set('destination', dir)
  }

  const submit = async (e: FormEvent) => {
    e.preventDefault()
    const rule = { ...draft, name: draft.name.trim() }
    const problem = inTauri() ? await api.validateRule(rule) : null
    if (problem) {
      setError(problem)
      return
    }
    onSave(rule)
  }

  return (
        <form onSubmit={submit} className="grid gap-4">
          <DialogHeader>
            <DialogTitle>{isNew ? t('rules.newTitle') : t('rules.editTitle')}</DialogTitle>
            <DialogDescription className="sr-only">{t('rules.subtitle')}</DialogDescription>
          </DialogHeader>

          <div className="grid gap-2">
            <Label htmlFor="rule-name">{t('rules.name')}</Label>
            <Input
              id="rule-name"
              value={draft.name}
              placeholder={t('rules.namePlaceholder')}
              onChange={(e) => set('name', e.target.value)}
              required
              autoFocus
            />
          </div>

          <div className="grid gap-2">
            <Label>{t('rules.patternType')}</Label>
            <RadioGroup
              value={draft.kind}
              onValueChange={(v) => set('kind', v as PatternKind)}
              className="flex gap-6"
            >
              {(['glob', 'regex'] as const).map((k) => (
                <div key={k} className="flex items-center gap-2">
                  <RadioGroupItem value={k} id={`kind-${k}`} />
                  <Label htmlFor={`kind-${k}`} className="font-normal">
                    {t(`rules.${k}`)}
                  </Label>
                </div>
              ))}
            </RadioGroup>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="rule-pattern">{t('rules.pattern')}</Label>
            <Input
              id="rule-pattern"
              value={draft.pattern}
              onChange={(e) => set('pattern', e.target.value)}
              className="font-mono"
              spellCheck={false}
              autoCapitalize="off"
              autoCorrect="off"
              required
            />
            <p className="text-muted-foreground text-xs">
              {draft.kind === 'glob' ? t('rules.globHelp') : t('rules.regexHelp')}
            </p>
          </div>

          <div className="flex items-center gap-2">
            <Switch
              id="rule-case"
              checked={draft.caseSensitive}
              onCheckedChange={(v) => set('caseSensitive', v)}
            />
            <Label htmlFor="rule-case" className="font-normal">
              {t('rules.caseSensitive')}
            </Label>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="rule-dest">{t('rules.destination')}</Label>
            <div className="flex gap-2">
              <Input
                id="rule-dest"
                value={draft.destination}
                placeholder={t('rules.destinationPlaceholder')}
                onChange={(e) => set('destination', e.target.value)}
                required
              />
              <Button type="button" variant="outline" onClick={browse} disabled={!inTauri()}>
                <FolderOpen data-icon="inline-start" />
                {t('common.browse')}
              </Button>
            </div>
          </div>

          {error && (
            <p role="alert" className="text-destructive text-sm">
              {error}
            </p>
          )}

          <DialogFooter>
            <Button type="button" variant="outline" onClick={onClose}>
              {t('common.cancel')}
            </Button>
            <Button type="submit">{t('common.save')}</Button>
          </DialogFooter>
        </form>
  )
}
