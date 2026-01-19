/**
 * useTimer Hook Tests - TDD Compliant
 * 
 * Following Red-Green-Refactor cycle:
 * 1. Write failing test (RED)
 * 2. Verify it fails for the right reason
 * 3. Write minimal code to pass (GREEN)
 * 4. Refactor while staying green
 */

import { renderHook, act, waitFor } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { useTimer } from '../useTimer'

// Mock the persistence module
vi.mock('@/lib/pomodoroPersistence', () => ({
    loadPomodoroState: vi.fn(() => ({
        version: 1,
        workMinutes: 25,
        breakMinutes: 5,
        mode: 'work' as const,
        state: 'idle' as const,
        timeLeft: 25 * 60,
        cycles: 0,
        lastUpdatedAt: Date.now(),
    })),
    savePomodoroState: vi.fn(),
    clearPomodoroRuntime: vi.fn(),
}))

// Mock window notifications as a proper constructor
class MockNotification {
    static permission = 'granted'
    constructor(public title: string, public options?: NotificationOptions) { }
}
vi.stubGlobal('Notification', MockNotification)

describe('useTimer', () => {
    beforeEach(() => {
        vi.useFakeTimers()
        vi.clearAllMocks()
    })

    afterEach(() => {
        vi.useRealTimers()
    })

    describe('Initial State', () => {
        it('returns idle state with default work duration', () => {
            const { result } = renderHook(() => useTimer())

            expect(result.current.state).toBe('idle')
            expect(result.current.mode).toBe('work')
            expect(result.current.workMinutes).toBe(25)
            expect(result.current.breakMinutes).toBe(5)
            expect(result.current.timeLeft).toBe(25 * 60)
            expect(result.current.cycles).toBe(0)
        })

        it('displays formatted time as MM:SS', () => {
            const { result } = renderHook(() => useTimer())

            expect(result.current.display).toBe('25:00')
        })

        it('starts with 0% progress', () => {
            const { result } = renderHook(() => useTimer())

            expect(result.current.progress).toBe(0)
        })
    })

    describe('Start Timer', () => {
        it('transitions to running state when start is called', () => {
            const { result } = renderHook(() => useTimer())

            act(() => {
                result.current.start()
            })

            expect(result.current.state).toBe('running')
            expect(result.current.mode).toBe('work')
        })

        it('decrements timeLeft every second while running', async () => {
            const { result } = renderHook(() => useTimer())

            act(() => {
                result.current.start()
            })

            const initialTimeLeft = result.current.timeLeft

            act(() => {
                vi.advanceTimersByTime(1000)
            })

            expect(result.current.timeLeft).toBe(initialTimeLeft - 1)
        })

        it('updates display after time passes', () => {
            const { result } = renderHook(() => useTimer())

            act(() => {
                result.current.start()
            })

            act(() => {
                vi.advanceTimersByTime(61000) // 1 minute and 1 second
            })

            expect(result.current.display).toBe('23:59')
        })
    })

    describe('Pause and Resume', () => {
        it('stops decrementing when paused', () => {
            const { result } = renderHook(() => useTimer())

            act(() => {
                result.current.start()
            })

            act(() => {
                vi.advanceTimersByTime(5000)
            })

            const timeBeforePause = result.current.timeLeft

            act(() => {
                result.current.pause()
            })

            expect(result.current.state).toBe('paused')

            act(() => {
                vi.advanceTimersByTime(5000)
            })

            expect(result.current.timeLeft).toBe(timeBeforePause)
        })

        it('resumes counting from paused time', () => {
            const { result } = renderHook(() => useTimer())

            act(() => {
                result.current.start()
            })

            act(() => {
                vi.advanceTimersByTime(5000)
            })

            act(() => {
                result.current.pause()
            })

            const pausedTime = result.current.timeLeft

            act(() => {
                result.current.resume()
            })

            expect(result.current.state).toBe('running')

            act(() => {
                vi.advanceTimersByTime(2000)
            })

            expect(result.current.timeLeft).toBe(pausedTime - 2)
        })
    })

    describe('Reset', () => {
        it('resets to idle state with initial values', async () => {
            const { result } = renderHook(() => useTimer())

            act(() => {
                result.current.start()
            })

            act(() => {
                vi.advanceTimersByTime(10000)
            })

            act(() => {
                result.current.reset()
            })

            expect(result.current.state).toBe('idle')
            expect(result.current.timeLeft).toBe(25 * 60)
            expect(result.current.cycles).toBe(0)
        })
    })

    describe('Skip', () => {
        it('skips from work to break and increments cycles', () => {
            const { result } = renderHook(() => useTimer())

            act(() => {
                result.current.start()
            })

            act(() => {
                result.current.skip()
            })

            expect(result.current.mode).toBe('break')
            expect(result.current.cycles).toBe(1)
            expect(result.current.timeLeft).toBe(5 * 60)
        })

        it('skips from break to work without incrementing cycles', () => {
            const { result } = renderHook(() => useTimer())

            act(() => {
                result.current.start()
            })

            // Skip to break
            act(() => {
                result.current.skip()
            })

            const cyclesAfterWork = result.current.cycles

            // Skip back to work
            act(() => {
                result.current.skip()
            })

            expect(result.current.mode).toBe('work')
            expect(result.current.cycles).toBe(cyclesAfterWork) // Should not increment
            expect(result.current.timeLeft).toBe(25 * 60)
        })
    })

    describe('Phase Completion', () => {
        it('calls onComplete when work session finishes', () => {
            const onComplete = vi.fn()
            const { result } = renderHook(() => useTimer({ onComplete }))

            act(() => {
                result.current.start()
            })

            // Fast-forward through entire work session
            act(() => {
                vi.advanceTimersByTime(25 * 60 * 1000)
            })

            expect(onComplete).toHaveBeenCalledWith('work')
        })

        it('transitions to break after work completes', () => {
            const { result } = renderHook(() => useTimer())

            act(() => {
                result.current.start()
            })

            act(() => {
                vi.advanceTimersByTime(25 * 60 * 1000)
            })

            expect(result.current.mode).toBe('break')
            expect(result.current.timeLeft).toBe(5 * 60)
        })

        it('increments cycle count when work session completes', () => {
            const { result } = renderHook(() => useTimer())

            act(() => {
                result.current.start()
            })

            expect(result.current.cycles).toBe(0)

            act(() => {
                vi.advanceTimersByTime(25 * 60 * 1000)
            })

            expect(result.current.cycles).toBe(1)
        })
    })

    describe('Duration Configuration', () => {
        it('allows setting work minutes with validation', () => {
            const { result } = renderHook(() => useTimer())

            act(() => {
                result.current.setWorkMinutes(45)
            })

            expect(result.current.workMinutes).toBe(45)
        })

        it('clamps work minutes to valid range (1-120)', () => {
            const { result } = renderHook(() => useTimer())

            act(() => {
                result.current.setWorkMinutes(200)
            })

            expect(result.current.workMinutes).toBe(120)

            act(() => {
                result.current.setWorkMinutes(0)
            })

            expect(result.current.workMinutes).toBe(1)
        })

        it('clamps break minutes to valid range (1-60)', () => {
            const { result } = renderHook(() => useTimer())

            act(() => {
                result.current.setBreakMinutes(100)
            })

            expect(result.current.breakMinutes).toBe(60)

            act(() => {
                result.current.setBreakMinutes(-5)
            })

            expect(result.current.breakMinutes).toBe(1)
        })
    })

    describe('Progress Calculation', () => {
        it('calculates progress as percentage of elapsed time', () => {
            const { result } = renderHook(() => useTimer())

            act(() => {
                result.current.start()
            })

            // After half the work time
            act(() => {
                vi.advanceTimersByTime((25 * 60 * 1000) / 2)
            })

            expect(result.current.progress).toBeCloseTo(50, 0)
        })
    })
})
