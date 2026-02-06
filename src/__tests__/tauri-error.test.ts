import { describe, it, expect } from 'vitest'

import { decodeApiError, getApiErrorMessage } from '@/lib/tauri'

describe('tauri error decoding', () => {
  it('decodes api error payloads', () => {
    const error = {
      code: 'validation',
      message: 'Title cannot be empty',
      details: { field: 'title' },
    }

    expect(decodeApiError(error)).toEqual({
      code: 'validation',
      message: 'Title cannot be empty',
      details: { field: 'title' },
    })
  })

  it('maps transient errors to a consistent message', () => {
    const error = { code: 'transient', message: 'Database temporarily unavailable' }
    expect(getApiErrorMessage(error)).toBe('Temporary issue. Please try again.')
  })

  it('falls back to a generic message for unknown errors', () => {
    expect(getApiErrorMessage(null)).toBe('Something went wrong. Please try again.')
  })
})
