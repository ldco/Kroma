/**
 * Seed Missing Translations
 *
 * Adds translations for all locales (en, ru, he) to existing content.
 * Run with: npx tsx server/database/seed-translations.ts
 */
import Database from 'better-sqlite3'
import { drizzle } from 'drizzle-orm/better-sqlite3'
import * as schema from './schema'
import { existsSync } from 'fs'

const DB_PATH = process.env.DATABASE_URL || './data/sqlite.db'

if (!existsSync(DB_PATH)) {
  console.error('❌ Database not found. Run npm run db:seed first.')
  process.exit(1)
}

const sqlite = new Database(DB_PATH)
sqlite.pragma('journal_mode = WAL')
const db = drizzle(sqlite, { schema })

function upsertTranslation(locale: string, key: string, value: string) {
  sqlite.prepare(`INSERT OR REPLACE INTO translations (locale, key, value, updated_at) VALUES (?, ?, ?, ?)`).run(
    locale, key, value, Date.now()
  )
}

async function seedTranslations() {
  console.log('🌍 Seeding missing translations...\n')

  // ═══════════════════════════════════════════════════════════════════════════
  // FEATURES
  // ═══════════════════════════════════════════════════════════════════════════
  console.log('✨ Features...')
  const features = db.select().from(schema.features).all()

  const featureTranslations: Record<string, { en: { title: string; description: string }; ru: { title: string; description: string }; he: { title: string; description: string } }> = {
    'responsive-design': {
      en: { title: 'Responsive Design', description: 'Beautiful layouts that adapt perfectly to any screen size and device.' },
      ru: { title: 'Адаптивный дизайн', description: 'Красивые макеты, которые идеально адаптируются к любому размеру экрана.' },
      he: { title: 'עיצוב רספונסיבי', description: 'עיצובים יפים שמתאימים בצורה מושלמת לכל גודל מסך.' }
    },
    'fast-performance': {
      en: { title: 'Fast Performance', description: 'Lightning-fast loading speeds for the best user experience.' },
      ru: { title: 'Высокая производительность', description: 'Молниеносная скорость загрузки для лучшего пользовательского опыта.' },
      he: { title: 'ביצועים מהירים', description: 'מהירות טעינה מהירה כברק לחוויית משתמש מיטבית.' }
    },
    'seo-optimized': {
      en: { title: 'SEO Optimized', description: 'Built-in SEO best practices to help your site rank higher.' },
      ru: { title: 'SEO оптимизация', description: 'Встроенные лучшие практики SEO для повышения рейтинга сайта.' },
      he: { title: 'אופטימיזציה ל-SEO', description: 'שיטות עבודה מומלצות מובנות לקידום אתרים.' }
    },
    'secure-hosting': {
      en: { title: 'Secure Hosting', description: 'Enterprise-grade security to protect your data and users.' },
      ru: { title: 'Безопасный хостинг', description: 'Безопасность корпоративного уровня для защиты данных.' },
      he: { title: 'אירוח מאובטח', description: 'אבטחה ברמה ארגונית להגנה על הנתונים שלך.' }
    },
    'analytics-dashboard': {
      en: { title: 'Analytics Dashboard', description: 'Comprehensive analytics to track your business metrics.' },
      ru: { title: 'Панель аналитики', description: 'Комплексная аналитика для отслеживания бизнес-метрик.' },
      he: { title: 'לוח בקרה אנליטי', description: 'אנליטיקה מקיפה למעקב אחר המדדים העסקיים שלך.' }
    },
    '24-7-support': {
      en: { title: '24/7 Support', description: 'Round-the-clock expert support whenever you need it.' },
      ru: { title: 'Поддержка 24/7', description: 'Круглосуточная экспертная поддержка когда вам нужно.' },
      he: { title: 'תמיכה 24/7', description: 'תמיכה מקצועית מסביב לשעון בכל עת שתצטרך.' }
    }
  }

  for (const feature of features) {
    const trans = featureTranslations[feature.slug]
    if (trans) {
      for (const locale of ['en', 'ru', 'he'] as const) {
        upsertTranslation(locale, `feature.${feature.id}.title`, trans[locale].title)
        upsertTranslation(locale, `feature.${feature.id}.description`, trans[locale].description)
      }
      console.log(`   ✓ ${feature.slug}`)
    }
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // FAQ
  // ═══════════════════════════════════════════════════════════════════════════
  console.log('\n❓ FAQ...')
  const faqItems = db.select().from(schema.faqItems).all()

  const faqTranslations: Record<string, { en: { q: string; a: string }; ru: { q: string; a: string }; he: { q: string; a: string } }> = {
    'how-to-get-started': {
      en: { q: 'How do I get started?', a: 'Getting started is easy! Simply contact us through our form or email, and we\'ll schedule a free consultation to discuss your project needs.' },
      ru: { q: 'Как начать работу?', a: 'Начать легко! Просто свяжитесь с нами через форму или email, и мы назначим бесплатную консультацию для обсуждения вашего проекта.' },
      he: { q: 'איך מתחילים?', a: 'להתחיל זה קל! פשוט צרו איתנו קשר דרך הטופס או האימייל, ונקבע התייעצות חינם לדיון בצרכי הפרויקט שלכם.' }
    },
    'pricing-structure': {
      en: { q: 'What is your pricing structure?', a: 'We offer flexible pricing options including fixed-price projects and hourly rates. Each project is quoted individually based on scope and requirements.' },
      ru: { q: 'Какова ваша ценовая политика?', a: 'Мы предлагаем гибкие варианты ценообразования, включая фиксированную цену и почасовую оплату. Каждый проект оценивается индивидуально.' },
      he: { q: 'מהו מבנה התמחור שלכם?', a: 'אנו מציעים אפשרויות תמחור גמישות כולל מחיר קבוע ותעריף שעתי. כל פרויקט מתומחר בנפרד.' }
    },
    'project-timeline': {
      en: { q: 'How long does a typical project take?', a: 'Project timelines vary based on complexity. A simple website might take 2-4 weeks, while more complex applications can take 2-3 months or more.' },
      ru: { q: 'Сколько времени занимает типичный проект?', a: 'Сроки зависят от сложности. Простой сайт может занять 2-4 недели, более сложные приложения — 2-3 месяца и больше.' },
      he: { q: 'כמה זמן לוקח פרויקט טיפוסי?', a: 'לוחות זמנים משתנים לפי מורכבות. אתר פשוט עשוי לקחת 2-4 שבועות, יישומים מורכבים יותר 2-3 חודשים.' }
    },
    'support-maintenance': {
      en: { q: 'Do you offer ongoing support and maintenance?', a: 'Yes! We offer various support packages to keep your site running smoothly, including security updates, content updates, and performance monitoring.' },
      ru: { q: 'Предлагаете ли вы постоянную поддержку?', a: 'Да! Мы предлагаем различные пакеты поддержки, включая обновления безопасности, контента и мониторинг производительности.' },
      he: { q: 'האם אתם מציעים תמיכה ותחזוקה שוטפת?', a: 'כן! אנו מציעים חבילות תמיכה שונות כולל עדכוני אבטחה, עדכוני תוכן וניטור ביצועים.' }
    },
    'technologies-used': {
      en: { q: 'What technologies do you use?', a: 'We specialize in modern web technologies including Vue.js, Nuxt, React, Node.js, and various database solutions. We choose the best stack for each project.' },
      ru: { q: 'Какие технологии вы используете?', a: 'Мы специализируемся на современных технологиях: Vue.js, Nuxt, React, Node.js и различных базах данных. Выбираем лучший стек для каждого проекта.' },
      he: { q: 'באילו טכנולוגיות אתם משתמשים?', a: 'אנו מתמחים בטכנולוגיות מודרניות כולל Vue.js, Nuxt, React, Node.js ופתרונות מסדי נתונים. אנו בוחרים את הסטאק הטוב ביותר לכל פרויקט.' }
    }
  }

  for (const item of faqItems) {
    const trans = faqTranslations[item.slug]
    if (trans) {
      for (const locale of ['en', 'ru', 'he'] as const) {
        upsertTranslation(locale, `faq.${item.id}.question`, trans[locale].q)
        upsertTranslation(locale, `faq.${item.id}.answer`, trans[locale].a)
      }
      console.log(`   ✓ ${item.slug}`)
    }
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // TESTIMONIALS
  // ═══════════════════════════════════════════════════════════════════════════
  console.log('\n💬 Testimonials...')
  const testimonials = db.select().from(schema.testimonials).all()

  const testimonialQuotes: Record<string, { en: string; ru: string; he: string }> = {
    'John Smith': {
      en: 'Working with this team has been an absolute pleasure. They delivered our project on time and exceeded our expectations.',
      ru: 'Работа с этой командой была настоящим удовольствием. Они сдали проект вовремя и превзошли наши ожидания.',
      he: 'העבודה עם הצוות הזה הייתה תענוג מוחלט. הם סיפקו את הפרויקט בזמן ועלו על הציפיות שלנו.'
    },
    'Lisa Anderson': {
      en: 'The attention to detail and creative solutions they provided helped us increase our conversion rate by 40%.',
      ru: 'Внимание к деталям и креативные решения помогли нам увеличить конверсию на 40%.',
      he: 'תשומת הלב לפרטים והפתרונות היצירתיים עזרו לנו להגדיל את שיעור ההמרה ב-40%.'
    },
    'David Park': {
      en: 'Professional, responsive, and truly talented. I highly recommend their services to anyone looking for quality web development.',
      ru: 'Профессиональные, отзывчивые и действительно талантливые. Рекомендую их услуги всем.',
      he: 'מקצועיים, רספונסיביים ומוכשרים באמת. אני ממליץ בחום על השירותים שלהם.'
    }
  }

  for (const testimonial of testimonials) {
    const quotes = testimonialQuotes[testimonial.authorName]
    if (quotes) {
      for (const locale of ['en', 'ru', 'he'] as const) {
        upsertTranslation(locale, `testimonial.${testimonial.id}.quote`, quotes[locale])
      }
      console.log(`   ✓ ${testimonial.authorName}`)
    }
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // TEAM
  // ═══════════════════════════════════════════════════════════════════════════
  console.log('\n👥 Team...')
  const teamMembers = db.select().from(schema.teamMembers).all()

  const teamTranslations: Record<string, { en: { position: string; bio: string }; ru: { position: string; bio: string }; he: { position: string; bio: string } }> = {
    'alex-johnson': {
      en: { position: 'CEO & Founder', bio: 'With over 15 years of experience in software development, Alex leads our company vision and strategy.' },
      ru: { position: 'Генеральный директор', bio: 'С более чем 15-летним опытом в разработке ПО, Алекс руководит видением и стратегией компании.' },
      he: { position: 'מנכ"ל ומייסד', bio: 'עם יותר מ-15 שנות ניסיון בפיתוח תוכנה, אלכס מוביל את החזון והאסטרטגיה של החברה.' }
    },
    'sarah-chen': {
      en: { position: 'CTO', bio: 'Sarah brings deep technical expertise and innovation to our engineering practices.' },
      ru: { position: 'Технический директор', bio: 'Сара привносит глубокую техническую экспертизу и инновации в нашу инженерную практику.' },
      he: { position: 'סמנכ"ל טכנולוגיות', bio: 'שרה מביאה מומחיות טכנית עמוקה וחדשנות לתהליכי ההנדסה שלנו.' }
    },
    'mike-wilson': {
      en: { position: 'Lead Designer', bio: 'Mike crafts beautiful user experiences with a keen eye for detail and usability.' },
      ru: { position: 'Ведущий дизайнер', bio: 'Майк создаёт красивый пользовательский опыт с вниманием к деталям и удобству.' },
      he: { position: 'מעצב ראשי', bio: 'מייק יוצר חוויות משתמש יפות עם עין חדה לפרטים ושימושיות.' }
    },
    'emma-davis': {
      en: { position: 'Marketing Director', bio: 'Emma drives our brand strategy and customer engagement initiatives.' },
      ru: { position: 'Директор по маркетингу', bio: 'Эмма руководит стратегией бренда и инициативами по работе с клиентами.' },
      he: { position: 'מנהלת שיווק', bio: 'אמה מובילה את אסטרטגיית המותג ויוזמות מעורבות הלקוחות שלנו.' }
    }
  }

  for (const member of teamMembers) {
    const trans = teamTranslations[member.slug]
    if (trans) {
      for (const locale of ['en', 'ru', 'he'] as const) {
        upsertTranslation(locale, `team.${member.id}.position`, trans[locale].position)
        upsertTranslation(locale, `team.${member.id}.bio`, trans[locale].bio)
      }
      console.log(`   ✓ ${member.name}`)
    }
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // PRICING TIERS
  // ═══════════════════════════════════════════════════════════════════════════
  console.log('\n💰 Pricing Tiers...')
  const pricingTiers = db.select().from(schema.pricingTiers).all()

  const pricingTierTranslations: Record<string, { en: { name: string; description: string; cta: string }; ru: { name: string; description: string; cta: string }; he: { name: string; description: string; cta: string } }> = {
    'starter': {
      en: { name: 'Starter', description: 'Perfect for small projects', cta: 'Get Started' },
      ru: { name: 'Стартовый', description: 'Идеально для небольших проектов', cta: 'Начать' },
      he: { name: 'התחלתי', description: 'מושלם לפרויקטים קטנים', cta: 'התחל עכשיו' }
    },
    'pro': {
      en: { name: 'Pro', description: 'For growing businesses', cta: 'Start Free Trial' },
      ru: { name: 'Профессиональный', description: 'Для растущего бизнеса', cta: 'Попробовать бесплатно' },
      he: { name: 'מקצועי', description: 'לעסקים בצמיחה', cta: 'התחל ניסיון חינם' }
    },
    'enterprise': {
      en: { name: 'Enterprise', description: 'Custom solutions', cta: 'Contact Sales' },
      ru: { name: 'Корпоративный', description: 'Индивидуальные решения', cta: 'Связаться с нами' },
      he: { name: 'ארגוני', description: 'פתרונות מותאמים', cta: 'צור קשר' }
    }
  }

  for (const tier of pricingTiers) {
    const trans = pricingTierTranslations[tier.slug]
    if (trans) {
      for (const locale of ['en', 'ru', 'he'] as const) {
        upsertTranslation(locale, `pricing.tier.${tier.id}.name`, trans[locale].name)
        upsertTranslation(locale, `pricing.tier.${tier.id}.description`, trans[locale].description)
        upsertTranslation(locale, `pricing.tier.${tier.id}.ctaText`, trans[locale].cta)
      }
      console.log(`   ✓ ${tier.slug}`)
    }
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // PRICING FEATURES
  // ═══════════════════════════════════════════════════════════════════════════
  console.log('\n📋 Pricing Features...')
  const pricingFeatures = db.select().from(schema.pricingFeatures).all()

  const pricingFeatureTranslations: Record<string, { en: string; ru: string; he: string }> = {
    'Up to 3 pages': { en: 'Up to 3 pages', ru: 'До 3 страниц', he: 'עד 3 עמודים' },
    'Basic blocks': { en: 'Basic blocks', ru: 'Базовые блоки', he: 'בלוקים בסיסיים' },
    'Community support': { en: 'Community support', ru: 'Поддержка сообщества', he: 'תמיכת קהילה' },
    'Visual editor': { en: 'Visual editor', ru: 'Визуальный редактор', he: 'עורך ויזואלי' },
    'Custom modules': { en: 'Custom modules', ru: 'Кастомные модули', he: 'מודולים מותאמים' },
    'Unlimited pages': { en: 'Unlimited pages', ru: 'Неограниченное количество страниц', he: 'עמודים ללא הגבלה' },
    'All blocks': { en: 'All blocks', ru: 'Все блоки', he: 'כל הבלוקים' },
    'Priority support': { en: 'Priority support', ru: 'Приоритетная поддержка', he: 'תמיכה בעדיפות' },
    'Dedicated support': { en: 'Dedicated support', ru: 'Персональная поддержка', he: 'תמיכה ייעודית' }
  }

  for (const feature of pricingFeatures) {
    const trans = pricingFeatureTranslations[feature.text]
    if (trans) {
      for (const locale of ['en', 'ru', 'he'] as const) {
        upsertTranslation(locale, `pricing.feature.${feature.id}.text`, trans[locale])
      }
      console.log(`   ✓ ${feature.text}`)
    }
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // BLOG CATEGORIES
  // ═══════════════════════════════════════════════════════════════════════════
  console.log('\n📂 Blog Categories...')
  const blogCategories = db.select().from(schema.blogCategories).all()

  const categoryTranslations: Record<string, { en: string; ru: string; he: string }> = {
    'technology': { en: 'Technology', ru: 'Технологии', he: 'טכנולוגיה' },
    'design': { en: 'Design', ru: 'Дизайн', he: 'עיצוב' },
    'business': { en: 'Business', ru: 'Бизнес', he: 'עסקים' },
    'tutorials': { en: 'Tutorials', ru: 'Уроки', he: 'מדריכים' }
  }

  for (const cat of blogCategories) {
    const trans = categoryTranslations[cat.slug]
    if (trans) {
      for (const locale of ['en', 'ru', 'he'] as const) {
        upsertTranslation(locale, `blog.category.${cat.id}.name`, trans[locale])
      }
      console.log(`   ✓ ${cat.slug}`)
    }
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // BLOG POSTS
  // ═══════════════════════════════════════════════════════════════════════════
  console.log('\n📝 Blog Posts...')
  const blogPosts = db.select().from(schema.blogPosts).all()

  const postTranslations: Record<string, { en: { title: string; excerpt: string; content: string }; ru: { title: string; excerpt: string; content: string }; he: { title: string; excerpt: string; content: string } }> = {
    'getting-started-with-nuxt3': {
      en: { title: 'Getting Started with Nuxt 3', excerpt: 'Learn the basics of Nuxt 3 and how to build modern web applications.', content: '## Introduction\n\nNuxt 3 is a powerful framework for building Vue.js applications...\n\n## Getting Started\n\nFirst, create a new project using the following command...' },
      ru: { title: 'Начало работы с Nuxt 3', excerpt: 'Изучите основы Nuxt 3 и создавайте современные веб-приложения.', content: '## Введение\n\nNuxt 3 — мощный фреймворк для создания приложений на Vue.js...\n\n## Начало работы\n\nСначала создайте новый проект с помощью команды...' },
      he: { title: 'מתחילים עם Nuxt 3', excerpt: 'למדו את הבסיס של Nuxt 3 וכיצד לבנות אפליקציות ווב מודרניות.', content: '## הקדמה\n\nNuxt 3 הוא פריימוורק חזק לבניית אפליקציות Vue.js...\n\n## מתחילים\n\nראשית, צרו פרויקט חדש באמצעות הפקודה הבאה...' }
    },
    'modern-css-techniques': {
      en: { title: 'Modern CSS Techniques for 2024', excerpt: 'Explore the latest CSS features and best practices for modern web design.', content: '## CSS Has Evolved\n\nModern CSS offers powerful features like CSS Grid, Container Queries, and more...' },
      ru: { title: 'Современные техники CSS в 2024', excerpt: 'Изучите последние возможности CSS и лучшие практики современного веб-дизайна.', content: '## CSS развивается\n\nСовременный CSS предлагает мощные возможности: CSS Grid, Container Queries и многое другое...' },
      he: { title: 'טכניקות CSS מודרניות ל-2024', excerpt: 'גלו את התכונות האחרונות של CSS ושיטות עבודה מומלצות לעיצוב אתרים מודרני.', content: '## CSS התפתח\n\nCSS מודרני מציע תכונות חזקות כמו CSS Grid, Container Queries ועוד...' }
    },
    'building-scalable-apis': {
      en: { title: 'Building Scalable APIs with Node.js', excerpt: 'A comprehensive guide to creating performant and maintainable APIs.', content: '## API Design Principles\n\nWhen building APIs, consider these key principles...' },
      ru: { title: 'Создание масштабируемых API на Node.js', excerpt: 'Полное руководство по созданию производительных и поддерживаемых API.', content: '## Принципы проектирования API\n\nПри создании API учитывайте следующие ключевые принципы...' },
      he: { title: 'בניית APIs מתרחבים עם Node.js', excerpt: 'מדריך מקיף ליצירת APIs יעילים וניתנים לתחזוקה.', content: '## עקרונות עיצוב API\n\nבעת בניית APIs, שקלו את העקרונות המרכזיים הבאים...' }
    }
  }

  for (const post of blogPosts) {
    const trans = postTranslations[post.slug]
    if (trans) {
      for (const locale of ['en', 'ru', 'he'] as const) {
        upsertTranslation(locale, `blog.post.${post.id}.title`, trans[locale].title)
        upsertTranslation(locale, `blog.post.${post.id}.excerpt`, trans[locale].excerpt)
        upsertTranslation(locale, `blog.post.${post.id}.content`, trans[locale].content)
      }
      console.log(`   ✓ ${post.slug}`)
    }
  }

  console.log('\n✅ All translations seeded!\n')
  sqlite.close()
}

seedTranslations().catch(error => {
  console.error('❌ Seed failed:', error)
  sqlite.close()
  process.exit(1)
})
