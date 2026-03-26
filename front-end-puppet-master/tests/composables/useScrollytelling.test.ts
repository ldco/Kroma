import { describe, expect, it, vi, afterEach } from 'vitest'
import { defineComponent, h, nextTick } from 'vue'
import { mountSuspended } from '@nuxt/test-utils/runtime'
import { useScrollytelling } from '~/composables/useScrollytelling'

describe('useScrollytelling global timeline contract', () => {
  afterEach(() => {
    vi.restoreAllMocks()
    vi.unstubAllGlobals()
    document.documentElement.style.removeProperty('--pm-scroll-progress')
  })

  it('updates --pm-scroll-progress against full page scroll height', async () => {
    let scrollY = 0

    class MockMutationObserver {
      constructor(_callback: MutationCallback) {}
      observe() {}
      disconnect() {}
    }

    vi.stubGlobal('MutationObserver', MockMutationObserver)

    const scrollYSpy = vi.spyOn(window, 'scrollY', 'get').mockImplementation(() => scrollY)
    const innerHeightSpy = vi.spyOn(window, 'innerHeight', 'get').mockReturnValue(1000)
    const scrollHeightSpy = vi.spyOn(document.documentElement, 'scrollHeight', 'get').mockReturnValue(3000)
    const rafSpy = vi.spyOn(window, 'requestAnimationFrame').mockImplementation(callback => {
      Promise.resolve().then(() => callback(16))
      return 1
    })
    const cancelRafSpy = vi.spyOn(window, 'cancelAnimationFrame').mockImplementation(() => {})

    const Harness = defineComponent({
      setup() {
        useScrollytelling({ enabled: true })
        return () =>
          h(
            'div',
            { class: 'scene-full-bleed', 'data-scrolly-scene': 'true' },
            [h('section', { class: 'section' }, [h('div', { style: 'height: 200vh' })])]
          )
      }
    })

    const flushFrame = async () => {
      await Promise.resolve()
      await nextTick()
    }

    const wrapper = await mountSuspended(Harness)
    await flushFrame()

    const readProgress = () =>
      Number.parseFloat(document.documentElement.style.getPropertyValue('--pm-scroll-progress') || '0')

    // maxScroll = 3000 - 1000 = 2000
    expect(readProgress()).toBeCloseTo(0, 4)

    scrollY = 1000
    window.dispatchEvent(new Event('scroll'))
    await flushFrame()
    expect(readProgress()).toBeCloseTo(0.5, 2)

    scrollY = 2000
    window.dispatchEvent(new Event('scroll'))
    await flushFrame()
    expect(readProgress()).toBeCloseTo(1, 4)

    wrapper.unmount()

    scrollYSpy.mockRestore()
    innerHeightSpy.mockRestore()
    scrollHeightSpy.mockRestore()
    rafSpy.mockRestore()
    cancelRafSpy.mockRestore()
  })
})
