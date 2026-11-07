import { QueryClient, QueryObserver, focusManager } from '@tanstack/react-query'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { targetIconsQueryOptions } from './queries'

describe('target icons query', () => {
  afterEach(() => {
    focusManager.setFocused(undefined)
  })

  it('refetches on window focus even inside the staleTime window', async () => {
    let calls = 0
    const client = new QueryClient()
    // QueryClientProvider 在应用里做的就是这件事：没有 mount 就收不到焦点事件。
    client.mount()
    const observer = new QueryObserver(client, {
      queryKey: ['target-icons-focus-test'],
      queryFn: async () => {
        calls += 1
        return []
      },
      ...targetIconsQueryOptions,
    })
    const unsubscribe = observer.subscribe(() => {})

    await vi.waitFor(() => expect(calls).toBe(1))

    // 用户打开图标目录、放入新文件后切回应用。
    focusManager.setFocused(false)
    focusManager.setFocused(true)

    await vi.waitFor(() => expect(calls).toBe(2))

    unsubscribe()
    observer.destroy()
    client.clear()
    client.unmount()
  })
})
