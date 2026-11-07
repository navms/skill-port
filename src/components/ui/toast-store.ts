import { create } from 'zustand'

export type ToastTone = 'ok' | 'warn' | 'err'

export interface ToastAction {
  label: string
  onClick: () => void
}

export interface ToastItem {
  id: number
  message: string
  tone: ToastTone
  action?: ToastAction
}

interface ToastOptions {
  tone?: ToastTone
  action?: ToastAction
  duration?: number
}

interface ToastState {
  items: ToastItem[]
  push: (item: Omit<ToastItem, 'id'>) => number
  dismiss: (id: number) => void
}

const timers = new Map<number, ReturnType<typeof setTimeout>>()
let nextId = 1

function clearTimer(id: number): void {
  const timer = timers.get(id)
  if (timer) {
    clearTimeout(timer)
    timers.delete(id)
  }
}

export const useToastStore = create<ToastState>()((set, get) => ({
  items: [],
  push: (item) => {
    const id = nextId
    nextId += 1
    const items = [...get().items, { ...item, id }]
    const overflow = items.length > 3 ? items.slice(0, items.length - 3) : []
    for (const stale of overflow) clearTimer(stale.id)
    set({ items: items.slice(-3) })
    return id
  },
  dismiss: (id) => {
    clearTimer(id)
    set({ items: get().items.filter((item) => item.id !== id) })
  },
}))

export function toast(message: string, options: ToastOptions = {}): number {
  const { tone = 'ok', action, duration = action ? 6500 : 4000 } = options
  const id = useToastStore.getState().push({ message, tone, action })
  timers.set(
    id,
    setTimeout(() => {
      useToastStore.getState().dismiss(id)
    }, duration),
  )
  return id
}
