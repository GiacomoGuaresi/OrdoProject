import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'

import type { Lang } from '@/bindings/Lang'
import en from '@/locales/en'
import it from '@/locales/it'

export const LANGUAGES: Record<Lang, string> = { en: 'English', it: 'Italiano' }

void i18n.use(initReactI18next).init({
  resources: { en: { translation: en }, it: { translation: it } },
  lng: 'en',
  fallbackLng: 'en',
  interpolation: { escapeValue: false },
})

export default i18n
