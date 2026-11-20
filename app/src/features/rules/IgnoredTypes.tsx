import { Plus, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import type { IgnoredType } from '@/bindings/IgnoredType'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { formatDateTime } from '@/lib/format'

interface Props {
  types: IgnoredType[]
  onCreateRule: (type: IgnoredType) => void
  onDismiss: (type: IgnoredType) => void
  onClear: () => void
}

/** File extensions that no rule matched, to turn them into rules quickly. */
export function IgnoredTypes({ types, onCreateRule, onDismiss, onClear }: Props) {
  const { t, i18n } = useTranslation()
  if (types.length === 0) return null
  return (
    <section className="flex flex-col gap-3" aria-labelledby="ignored-title">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 id="ignored-title" className="text-sm font-medium">
            {t('rules.ignoredTitle')}
          </h2>
          <p className="text-muted-foreground text-xs">{t('rules.ignoredHelp')}</p>
        </div>
        <Button variant="ghost" size="sm" onClick={onClear}>
          {t('rules.clearIgnored')}
        </Button>
      </div>
      <ul className="flex flex-col gap-1.5">
        {types.map((type) => (
          <li key={type.ext} className="flex items-center gap-3 rounded-lg border border-dashed px-3 py-2">
            <Badge variant="secondary" className="font-mono">
              .{type.ext}
            </Badge>
            <div className="text-muted-foreground min-w-0 flex-1 truncate text-xs">
              {t('rules.ignoredFiles', { count: type.count })} ·{' '}
              {formatDateTime(type.lastSeen, i18n.language)} ·{' '}
              <span title={type.example}>{t('rules.ignoredExample', { name: type.example })}</span>
            </div>
            <Button variant="outline" size="sm" onClick={() => onCreateRule(type)}>
              <Plus data-icon="inline-start" />
              {t('rules.createRule')}
            </Button>
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={t('rules.dismiss')}
              title={t('rules.dismiss')}
              onClick={() => onDismiss(type)}
            >
              <X />
            </Button>
          </li>
        ))}
      </ul>
    </section>
  )
}
