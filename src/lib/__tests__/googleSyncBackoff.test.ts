import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'

import {
  GOOGLE_SYNC_BACKOFF,
  clearGoogleSyncFailures,
  getGoogleSyncBackoffState,
  recordGoogleSyncFailure,
} from '../googleSyncBackoff'

describe('google sync backoff', () => {
  beforeEach(() => {
    clearGoogleSyncFailures()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('returns no backoff when no failures', () => {
    const now = new Date(2026, 0, 1, 10, 0, 0).getTime()
    vi.setSystemTime(now)

    const state = getGoogleSyncBackoffState(now)
    expect(state.failureCount).toBe(0)
    expect(state.lastFailureAt).toBe(0)
    expect(state.backoffMs).toBe(0)
    expect(state.shouldSkip).toBe(false)
  })

  it('records failures and skips within base backoff window', () => {
    const now = new Date(2026, 0, 1, 10, 0, 0).getTime()
    vi.setSystemTime(now)

    recordGoogleSyncFailure(now)
    const within = getGoogleSyncBackoffState(
      now + GOOGLE_SYNC_BACKOFF.baseBackoffMs - 1,
    )
    expect(within.shouldSkip).toBe(true)

    const after = getGoogleSyncBackoffState(
      now + GOOGLE_SYNC_BACKOFF.baseBackoffMs + 1,
    )
    expect(after.shouldSkip).toBe(false)
  })

  it('backs off exponentially after consecutive failures', () => {
    const now = new Date(2026, 0, 1, 10, 0, 0).getTime()
    vi.setSystemTime(now)

    recordGoogleSyncFailure(now)
    const second = now + 1000
    recordGoogleSyncFailure(second)

    const within = getGoogleSyncBackoffState(
      second + GOOGLE_SYNC_BACKOFF.baseBackoffMs * 2 - 1,
    )
    expect(within.shouldSkip).toBe(true)

    const after = getGoogleSyncBackoffState(
      second + GOOGLE_SYNC_BACKOFF.baseBackoffMs * 2 + 1,
    )
    expect(after.shouldSkip).toBe(false)
  })
})
