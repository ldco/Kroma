import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

function readProjectFile(path: string): string {
  return readFileSync(resolve(process.cwd(), path), 'utf-8')
}

describe('Style system contracts', () => {
  it('defines heading utility classes', () => {
    const textCss = readProjectFile('app/assets/css/common/text.css')

    expect(textCss).toContain('.heading-xs')
    expect(textCss).toContain('.heading-sm')
    expect(textCss).toContain('.heading-md')
    expect(textCss).toContain('.heading-lg')
    expect(textCss).toContain('.heading-xl')
  })

  it('defines font-black token', () => {
    const typographyVars = readProjectFile('app/assets/css/typography/variables.css')
    expect(typographyVars).toContain('--font-black')
  })

  it('defines explicit stack and form-to-action spacing contracts', () => {
    const inputsCss = readProjectFile('app/assets/css/ui/forms/inputs.css')

    expect(inputsCss).toContain('.stack > * + *')
    expect(inputsCss).toContain('.form-grid + .page-actions')
    expect(inputsCss).toContain('.grid + .cta')
  })

  it('ships segmented control reusable styles', () => {
    const inputsCss = readProjectFile('app/assets/css/ui/forms/inputs.css')

    expect(inputsCss).toContain('.segmented-control')
    expect(inputsCss).toContain('.segmented-control__item')
  })

  it('keeps data-table readability defaults', () => {
    const dataTableCss = readProjectFile('app/assets/css/ui/content/data-table.css')

    expect(dataTableCss).toContain('table-layout: auto')
    expect(dataTableCss).toContain('.actions-col')
    expect(dataTableCss).toContain('min-width: 8.5rem')
  })

  it('keeps icon action hover contrast and button border contracts', () => {
    const buttonCss = readProjectFile('app/assets/css/ui/forms/buttons.css')

    expect(buttonCss).toContain('.btn-icon.btn-ghost:not(.btn-danger)')
    expect(buttonCss).toContain('.btn-primary')
    expect(buttonCss).toContain('border: 1px solid')
  })

  it('standardizes onepager scene wrapper contract via shared renderer', () => {
    const renderer = readProjectFile('app/components/organisms/SectionRenderer.vue')
    const sectionsCss = readProjectFile('app/assets/css/layout/sections.css')

    expect(renderer).toContain("'scene-full-bleed': config.features.onepager")
    expect(renderer).toContain('v-scrolly-scene')
    expect(sectionsCss).toContain('.onepager .scene-full-bleed')
    expect(sectionsCss).toContain('width: 100vw')
    expect(sectionsCss).toContain('max-width: var(--content-default)')
  })

  it('formalizes admin nav alignment contract by mode', () => {
    const adminHeaderCss = readProjectFile('app/assets/css/layout/admin-header.css')
    const sidebarNavCss = readProjectFile('app/assets/css/skeleton/nav.css')

    // Horizontal mode: centered nav cluster
    expect(adminHeaderCss).toContain('.admin-header-nav')
    expect(adminHeaderCss).toContain('justify-content: center')

    // Vertical mode: top-anchored navigation, footer pushed to bottom
    expect(sidebarNavCss).toContain('.sidebar-nav')
    expect(sidebarNavCss).toContain('flex: 1')
    expect(sidebarNavCss).toContain('.sidebar-footer')
    expect(sidebarNavCss).toContain('margin-block-start: auto')
  })
})
