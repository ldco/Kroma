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

  it('standardizes kroma sidebar navigation contract', () => {
    const sidebarCss = readProjectFile('app/assets/css/layout/kroma-sidebar.css')

    // Kroma sidebar: fixed position, flex column layout
    expect(sidebarCss).toContain('.kroma-sidebar')
    expect(sidebarCss).toContain('position: fixed')
    expect(sidebarCss).toContain('flex-direction: column')
    // Nav links have visible labels with icon+text layout
    expect(sidebarCss).toContain('.sidebar-nav-link')
    expect(sidebarCss).toContain('display: flex')
    expect(sidebarCss).toContain('.sidebar-nav-label')
  })

  it('formalizes admin nav alignment contract by mode', () => {
    const adminHeaderCss = readProjectFile('app/assets/css/layout/admin-header.css')
    const kromaSidebarCss = readProjectFile('app/assets/css/layout/kroma-sidebar.css')

    // Horizontal mode: centered nav cluster
    expect(adminHeaderCss).toContain('.admin-header-nav')
    expect(adminHeaderCss).toContain('justify-content: center')

    // Vertical mode (Kroma sidebar): top-anchored navigation, footer pushed to bottom
    expect(kromaSidebarCss).toContain('.sidebar-nav')
    expect(kromaSidebarCss).toContain('flex: 1')
    expect(kromaSidebarCss).toContain('.sidebar-footer')
  })
})
