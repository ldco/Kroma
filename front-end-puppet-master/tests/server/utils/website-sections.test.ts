import { describe, expect, it } from 'vitest'
import {
  buildWebsiteNavLinks,
  defineWebsiteSections,
  getDynamicWebsiteSectionIds,
  getNavWebsiteSectionIds,
  getWebsiteSectionHref
} from '../../../app/types/config'

describe('website section contracts', () => {
  it('validates and returns sections config', () => {
    const sections = defineWebsiteSections([
      { id: 'home', inNav: true },
      { id: 'about', inNav: true, props: { photoPlacement: 'outside-panel' } },
      { id: 'contact', inNav: false, props: { showMap: true } },
      { id: 'custom-section', inNav: true, props: { variant: 'compact' } }
    ] as const)

    expect(sections).toHaveLength(4)
  })

  it('rejects duplicate section ids', () => {
    expect(() =>
      defineWebsiteSections([
        { id: 'home', inNav: true },
        { id: 'about', inNav: true },
        { id: 'about', inNav: false }
      ] as const)
    ).toThrow('duplicate id "about"')
  })

  it('rejects non-kebab-case and whitespace ids', () => {
    expect(() =>
      defineWebsiteSections([
        { id: 'home', inNav: true },
        { id: 'About', inNav: true }
      ] as const)
    ).toThrow('lowercase-kebab-case')

    expect(() =>
      defineWebsiteSections([
        { id: 'home', inNav: true },
        { id: ' contact', inNav: true }
      ] as const)
    ).toThrow('leading/trailing whitespace')
  })

  it('requires home to be first when present', () => {
    expect(() =>
      defineWebsiteSections([
        { id: 'about', inNav: true },
        { id: 'home', inNav: true }
      ] as const)
    ).toThrow('"home" must be the first section')
  })

  it('derives nav ids and dynamic ids from the same source', () => {
    const sections = defineWebsiteSections([
      { id: 'home', inNav: true },
      { id: 'about', inNav: true },
      { id: 'faq', inNav: false },
      { id: 'contact', inNav: true }
    ] as const)

    expect(getNavWebsiteSectionIds(sections)).toEqual(['home', 'about', 'contact'])
    expect(getDynamicWebsiteSectionIds(sections)).toEqual(['about', 'faq', 'contact'])
  })

  it('builds onepager and SPA links consistently', () => {
    const sections = defineWebsiteSections([
      { id: 'home', inNav: true },
      { id: 'about', inNav: true },
      { id: 'contact', inNav: true }
    ] as const)

    const onepager = buildWebsiteNavLinks(sections, { onepager: true })
    expect(onepager).toEqual([
      { id: 'home', href: '#home', isAnchor: true },
      { id: 'about', href: '#about', isAnchor: true },
      { id: 'contact', href: '#contact', isAnchor: true }
    ])

    const spa = buildWebsiteNavLinks(sections, {
      onepager: false,
      localePath: (path: string) => `/en${path}`
    })
    expect(spa).toEqual([
      { id: 'home', href: '/en/', isAnchor: false },
      { id: 'about', href: '/en/about', isAnchor: false },
      { id: 'contact', href: '/en/contact', isAnchor: false }
    ])
  })

  it('computes section href for mode-specific routing', () => {
    expect(getWebsiteSectionHref('about', { onepager: true })).toBe('#about')
    expect(
      getWebsiteSectionHref('home', {
        onepager: false,
        localePath: (path: string) => `/ru${path}`
      })
    ).toBe('/ru/')
  })
})
