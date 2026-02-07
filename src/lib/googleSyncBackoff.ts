const FAILURE_KEY = 'lifeos:googleSyncLastFailureAt'
const COUNT_KEY = 'lifeos:googleSyncFailureCount'

export const GOOGLE_SYNC_BACKOFF = {
  baseBackoffMs: 30_000,
  maxBackoffMs: 10 * 60 * 1000,
}

type GoogleSyncBackoffState = {
  failureCount: number
  lastFailureAt: number
  backoffMs: number
  shouldSkip: boolean
}

const readNumber = (value: string | null): number | null => {
  if (!value) return null
  const parsed = Number(value)
  if (!Number.isFinite(parsed)) return null
  return parsed
}

const getBackoffMs = (failureCount: number): number => {
  if (failureCount <= 0) return 0
  const multiplier = Math.pow(2, failureCount - 1)
  return Math.min(GOOGLE_SYNC_BACKOFF.baseBackoffMs * multiplier, GOOGLE_SYNC_BACKOFF.maxBackoffMs)
}

export const getGoogleSyncBackoffState = (now = Date.now()): GoogleSyncBackoffState => {
  const lastFailureAt = readNumber(window.localStorage.getItem(FAILURE_KEY)) ?? 0
  const failureCount = readNumber(window.localStorage.getItem(COUNT_KEY)) ?? 0
  const backoffMs = getBackoffMs(failureCount)
  const shouldSkip = backoffMs > 0 && now - lastFailureAt < backoffMs
  return {
    failureCount,
    lastFailureAt,
    backoffMs,
    shouldSkip,
  }
}

export const recordGoogleSyncFailure = (now = Date.now()): void => {
  const currentCount = readNumber(window.localStorage.getItem(COUNT_KEY)) ?? 0
  const nextCount = currentCount + 1
  window.localStorage.setItem(COUNT_KEY, String(nextCount))
  window.localStorage.setItem(FAILURE_KEY, String(now))
}

export const clearGoogleSyncFailures = (): void => {
  window.localStorage.removeItem(FAILURE_KEY)
  window.localStorage.removeItem(COUNT_KEY)
}
