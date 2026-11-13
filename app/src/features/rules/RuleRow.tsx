import { useSortable } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { ArrowRight, GripVertical, Pencil, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import type { Rule } from '@/bindings/Rule'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import { cn } from '@/lib/utils'

interface Props {
  rule: Rule
  position: number
  onToggle: (enabled: boolean) => void
  onEdit: () => void
  onDelete: () => void
}

export function RuleRow({ rule, position, onToggle, onEdit, onDelete }: Props) {
  const { t } = useTranslation()
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: rule.id,
  })

  return (
    <li
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      className={cn(
        'bg-card flex items-center gap-3 rounded-lg border px-3 py-2.5',
        isDragging && 'relative z-10 shadow-lg',
        !rule.enabled && 'opacity-60',
      )}
    >
      <button
        type="button"
        className="text-muted-foreground hover:text-foreground cursor-grab touch-none active:cursor-grabbing"
        aria-label={t('rules.dragHandle')}
        {...attributes}
        {...listeners}
      >
        <GripVertical className="size-4" />
      </button>
      <span className="text-muted-foreground w-5 text-right text-xs tabular-nums">{position}</span>

      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="truncate font-medium">{rule.name}</span>
          <Badge variant="secondary" className="font-mono text-[0.7rem]">
            {t(`rules.${rule.kind}`)}
          </Badge>
          {rule.caseSensitive && (
            <Badge variant="outline" className="text-[0.7rem]" title={t('rules.caseSensitive')}>
              Aa
            </Badge>
          )}
        </div>
        <div className="text-muted-foreground flex min-w-0 items-center gap-1.5 text-xs">
          <code className="max-w-[50%] shrink-0 truncate font-mono" title={rule.pattern}>
            {rule.pattern}
          </code>
          <ArrowRight className="size-3 shrink-0" />
          <span className="min-w-0 truncate [direction:rtl] text-left" title={rule.destination}>
            <bdi dir="ltr">{rule.destination}</bdi>
          </span>
        </div>
      </div>

      <Switch
        checked={rule.enabled}
        onCheckedChange={onToggle}
        aria-label={rule.enabled ? t('rules.enabled') : t('rules.disabled')}
      />
      <Button variant="ghost" size="icon-sm" onClick={onEdit} aria-label={t('common.edit')}>
        <Pencil />
      </Button>
      <Button variant="ghost" size="icon-sm" onClick={onDelete} aria-label={t('common.delete')}>
        <Trash2 />
      </Button>
    </li>
  )
}
