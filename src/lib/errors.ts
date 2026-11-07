export interface AppErrorPayload {
  code: string
  message: string
  detail?: string | null
}

export class IpcError extends Error {
  readonly code: string
  readonly detail: string | null

  constructor(payload: AppErrorPayload) {
    super(payload.message)
    this.name = 'IpcError'
    this.code = payload.code
    this.detail = payload.detail ?? null
  }
}

export function isIpcError(error: unknown): error is IpcError {
  return error instanceof IpcError
}

export function toErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  return 'Unknown error'
}

export function toIpcError(error: unknown): IpcError {
  if (error instanceof IpcError) return error
  if (error && typeof error === 'object') {
    const candidate = error as Partial<AppErrorPayload>
    if (typeof candidate.code === 'string' && typeof candidate.message === 'string') {
      return new IpcError({
        code: candidate.code,
        message: candidate.message,
        detail: candidate.detail ?? null,
      })
    }
  }
  return new IpcError({
    code: 'Unknown',
    message: toErrorMessage(error),
    detail: null,
  })
}
