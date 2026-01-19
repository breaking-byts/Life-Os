/**
 * pomodoroPersistence.ts Library Tests - TDD Compliant
 * 
 * Testing state persistence, normalization, and validation
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import {
    loadPomodoroState,
    savePomodoroState,
    clearPomodoroState,
    clearPomodoroRuntime,
    pomodoroDefaults,
} from '../pomodoroPersistence'

describe('pomodoroPersistence', () => {
    beforeEach(() => {
        // Clear localStorage before each test (jsdom compatible)
        window.localStorage.removeItem('lifeos:pomodoro')
        vi.clearAllMocks()
    })

    describe('loadPomodoroState', () => {
        it('returns defaults when no stored state exists', () => {
            const state = loadPomodoroState()

            expect(state.workMinutes).toBe(pomodoroDefaults.workMinutes)
            expect(state.breakMinutes).toBe(pomodoroDefaults.breakMinutes)
            expect(state.mode).toBe('work')
            expect(state.state).toBe('idle')
            expect(state.cycles).toBe(0)
        })

        it('loads stored state correctly', () => {
            const stored = {
                version: 1,
                workMinutes: 30,
                breakMinutes: 10,
                mode: 'break',
                state: 'paused',
                timeLeft: 300,
                cycles: 3,
                selectedCourseId: 'course-1',
                activeSessionId: 42,
                lastUpdatedAt: Date.now(),
            }
            localStorage.setItem('lifeos:pomodoro', JSON.stringify(stored))

            const state = loadPomodoroState()

            expect(state.workMinutes).toBe(30)
            expect(state.breakMinutes).toBe(10)
            expect(state.mode).toBe('break')
            expect(state.state).toBe('paused')
            expect(state.cycles).toBe(3)
        })

        it('clamps work minutes to 1-120 range', () => {
            const stored = {
                version: 1,
                workMinutes: 200,
                breakMinutes: 5,
                mode: 'work',
                state: 'idle',
                timeLeft: 1500,
                cycles: 0,
                selectedCourseId: 'none',
                activeSessionId: null,
                lastUpdatedAt: Date.now(),
            }
            localStorage.setItem('lifeos:pomodoro', JSON.stringify(stored))

            const state = loadPomodoroState()
            expect(state.workMinutes).toBe(120)
        })

        it('clamps break minutes to 1-60 range', () => {
            const stored = {
                version: 1,
                workMinutes: 25,
                breakMinutes: 100,
                mode: 'work',
                state: 'idle',
                timeLeft: 1500,
                cycles: 0,
                selectedCourseId: 'none',
                activeSessionId: null,
                lastUpdatedAt: Date.now(),
            }
            localStorage.setItem('lifeos:pomodoro', JSON.stringify(stored))

            const state = loadPomodoroState()
            expect(state.breakMinutes).toBe(60)
        })

        it('normalizes invalid mode to work', () => {
            const stored = {
                version: 1,
                workMinutes: 25,
                breakMinutes: 5,
                mode: 'invalid',
                state: 'idle',
                timeLeft: 1500,
                cycles: 0,
                selectedCourseId: 'none',
                activeSessionId: null,
                lastUpdatedAt: Date.now(),
            }
            localStorage.setItem('lifeos:pomodoro', JSON.stringify(stored))

            const state = loadPomodoroState()
            expect(state.mode).toBe('work')
        })

        it('normalizes invalid state to idle', () => {
            const stored = {
                version: 1,
                workMinutes: 25,
                breakMinutes: 5,
                mode: 'work',
                state: 'invalid',
                timeLeft: 1500,
                cycles: 0,
                selectedCourseId: 'none',
                activeSessionId: null,
                lastUpdatedAt: Date.now(),
            }
            localStorage.setItem('lifeos:pomodoro', JSON.stringify(stored))

            const state = loadPomodoroState()
            expect(state.state).toBe('idle')
        })

        it('handles corrupted JSON gracefully', () => {
            localStorage.setItem('lifeos:pomodoro', 'not-valid-json')

            const state = loadPomodoroState()
            expect(state.workMinutes).toBe(pomodoroDefaults.workMinutes)
        })

        it('handles wrong version gracefully', () => {
            const stored = {
                version: 999,
                workMinutes: 50,
            }
            localStorage.setItem('lifeos:pomodoro', JSON.stringify(stored))

            const state = loadPomodoroState()
            expect(state.workMinutes).toBe(pomodoroDefaults.workMinutes)
        })
    })

    describe('savePomodoroState', () => {
        it('saves state to localStorage', () => {
            const state = {
                version: 1 as const,
                workMinutes: 30,
                breakMinutes: 10,
                mode: 'work' as const,
                state: 'running' as const,
                timeLeft: 1800,
                cycles: 2,
                selectedCourseId: 'course-abc',
                activeSessionId: 123,
                lastUpdatedAt: Date.now(),
            }

            savePomodoroState(state)

            const stored = localStorage.getItem('lifeos:pomodoro')
            expect(stored).not.toBeNull()

            const parsed = JSON.parse(stored!)
            expect(parsed.workMinutes).toBe(30)
            expect(parsed.cycles).toBe(2)
        })
    })

    describe('clearPomodoroState', () => {
        it('removes state from localStorage', () => {
            localStorage.setItem('lifeos:pomodoro', '{"version":1}')

            clearPomodoroState()

            expect(localStorage.getItem('lifeos:pomodoro')).toBeNull()
        })
    })

    describe('clearPomodoroRuntime', () => {
        it('resets runtime state while preserving settings', () => {
            const stored = {
                version: 1,
                workMinutes: 45,
                breakMinutes: 15,
                mode: 'break',
                state: 'running',
                timeLeft: 300,
                cycles: 5,
                selectedCourseId: 'course-1',
                activeSessionId: 99,
                lastUpdatedAt: Date.now() - 10000,
            }
            localStorage.setItem('lifeos:pomodoro', JSON.stringify(stored))

            clearPomodoroRuntime()

            const state = loadPomodoroState()

            // Settings preserved
            expect(state.workMinutes).toBe(45)
            expect(state.breakMinutes).toBe(15)
            expect(state.selectedCourseId).toBe('course-1')

            // Runtime reset
            expect(state.mode).toBe('work')
            expect(state.state).toBe('idle')
            expect(state.cycles).toBe(0)
            expect(state.activeSessionId).toBeNull()
            expect(state.timeLeft).toBe(45 * 60) // Based on workMinutes
        })
    })

    describe('pomodoroDefaults', () => {
        it('has expected default values', () => {
            expect(pomodoroDefaults.workMinutes).toBe(25)
            expect(pomodoroDefaults.breakMinutes).toBe(5)
            expect(pomodoroDefaults.selectedCourseId).toBe('none')
        })
    })
})
