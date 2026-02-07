import { vi } from 'vitest'

type MockResponse = Record<string, unknown>
const mockResponses: Map<string, MockResponse | MockResponse[]> = new Map()

/**
 * Mock Tauri invoke function that returns predefined responses
 */
export const mockInvoke = vi.fn(async (command: string, _args?: Record<string, unknown>) => {
    const response = mockResponses.get(command)
    if (response === undefined) {
        throw new Error(`No mock response registered for command: ${command}`)
    }

    // If array, shift to support sequential calls
    if (Array.isArray(response)) {
        const next = response.shift()
        if (response.length === 0) mockResponses.delete(command)
        return next
    }

    return response
})

/**
 * Register a mock response for a Tauri command
 */
export function setMockResponse(command: string, response: MockResponse | MockResponse[]) {
    mockResponses.set(command, response)
}

/**
 * Clear all mock responses
 */
export function clearMockResponses() {
    mockResponses.clear()
    mockInvoke.mockClear()
}

/**
 * Get all recorded invoke calls
 */
export function getInvokeCalls() {
    return mockInvoke.mock.calls
}

// Pre-built mock data factories
export const mockFactories = {
    course: (overrides = {}) => ({
        id: 1,
        name: 'Test Course',
        code: 'TEST101',
        credits: 3,
        color: '#3b82f6',
        semester: 'Spring 2026',
        instructor: 'Dr. Test',
        created_at: new Date().toISOString(),
        ...overrides,
    }),

    assignment: (overrides = {}) => ({
        id: 1,
        course_id: 1,
        title: 'Test Assignment',
        description: 'Test description',
        due_date: new Date(Date.now() + 86400000).toISOString(),
        completed: false,
        priority: 'medium',
        ...overrides,
    }),

    workout: (overrides = {}) => ({
        id: 1,
        user_id: 1,
        name: 'Test Workout',
        notes: '',
        duration_minutes: null,
        started_at: new Date().toISOString(),
        completed_at: null,
        ...overrides,
    }),

    session: (overrides = {}) => ({
        id: 1,
        skill_id: 1,
        started_at: new Date().toISOString(),
        ended_at: null,
        duration_minutes: null,
        notes: '',
        ...overrides,
    }),
}
