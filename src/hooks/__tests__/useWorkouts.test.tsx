/**
 * useWorkouts Hook Tests - TDD Compliant
 *
 * Tests for Workout CRUD operations:
 * - useWorkouts: main workout operations (create, update, delete, exercises)
 * - useWorkoutExercises: get exercises for a workout
 * - useWorkoutHeatmap: workout activity heatmap
 * - usePersonalRecords: PR tracking
 * - useWorkoutTemplates: template management
 *
 * Following Red-Green-Refactor cycle from test_driven_development.md skill
 */

import { renderHook, waitFor, act } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import {
    useWorkouts,
    useWorkoutExercises,
    useWorkoutHeatmap,
    usePersonalRecords,
    useWorkoutTemplates,
    useTemplateExercises,
} from '../useWorkouts'
import type { ReactNode } from 'react'

// Mock the Tauri invoke function
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}))

const mockedInvoke = vi.mocked(invoke)

// Test fixtures - real data structures
const mockWorkouts = [
    {
        id: 1,
        user_id: 1,
        name: 'Upper Body Day',
        notes: 'Focus on chest and shoulders',
        duration_minutes: 45,
        started_at: '2026-01-19T10:00:00Z',
        completed_at: '2026-01-19T10:45:00Z',
    },
    {
        id: 2,
        user_id: 1,
        name: 'Leg Day',
        notes: '',
        duration_minutes: null,
        started_at: '2026-01-18T14:00:00Z',
        completed_at: null,
    },
]

const mockWorkoutExercises = [
    {
        id: 1,
        workout_id: 1,
        exercise_id: 101,
        exercise_name: 'Bench Press',
        sets: 4,
        reps: 10,
        weight: 135,
        notes: 'Felt strong',
        order_index: 0,
    },
    {
        id: 2,
        workout_id: 1,
        exercise_id: 102,
        exercise_name: 'Shoulder Press',
        sets: 3,
        reps: 12,
        weight: 50,
        notes: '',
        order_index: 1,
    },
]

const mockHeatmapData = [
    { date: '2026-01-15', count: 1, duration_minutes: 45 },
    { date: '2026-01-17', count: 1, duration_minutes: 60 },
    { date: '2026-01-19', count: 2, duration_minutes: 90 },
]

const mockPersonalRecords = [
    {
        id: 1,
        exercise_id: 101,
        exercise_name: 'Bench Press',
        record_type: 'weight',
        value: 185.0,
        achieved_at: '2026-01-15T10:30:00Z',
    },
    {
        id: 2,
        exercise_id: 201,
        exercise_name: 'Squat',
        record_type: 'weight',
        value: 225.0,
        achieved_at: '2026-01-10T15:00:00Z',
    },
]

const mockTemplates = [
    {
        id: 1,
        name: 'Push Day',
        created_at: '2026-01-01T00:00:00Z',
    },
    {
        id: 2,
        name: 'Pull Day',
        created_at: '2026-01-01T00:00:00Z',
    },
]

const mockTemplateExercises = [
    {
        id: 1,
        template_id: 1,
        exercise_id: 101,
        exercise_name: 'Bench Press',
        default_sets: 4,
        default_reps: 10,
        order_index: 0,
    },
]

// Create wrapper with QueryClient
function createWrapper() {
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: {
                retry: false,
                gcTime: 0,
            },
        },
    })
    return function Wrapper({ children }: { children: ReactNode }) {
        return (
            <QueryClientProvider client={queryClient}>
                {children}
            </QueryClientProvider>
        )
    }
}

describe('useWorkouts', () => {
    beforeEach(() => {
        vi.clearAllMocks()
    })

    afterEach(() => {
        vi.resetAllMocks()
    })

    describe('Workouts Query', () => {
        it('fetches workouts on mount', async () => {
            mockedInvoke.mockResolvedValueOnce(mockWorkouts)

            const { result } = renderHook(() => useWorkouts(), {
                wrapper: createWrapper(),
            })

            expect(result.current.workoutsQuery.isLoading).toBe(true)

            await waitFor(() => {
                expect(result.current.workoutsQuery.isLoading).toBe(false)
            })

            expect(mockedInvoke).toHaveBeenCalledWith('get_workouts', undefined)
            expect(result.current.workoutsQuery.data).toEqual(mockWorkouts)
        })

        it('returns empty array when no workouts exist', async () => {
            mockedInvoke.mockResolvedValueOnce([])

            const { result } = renderHook(() => useWorkouts(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.workoutsQuery.isLoading).toBe(false)
            })

            expect(result.current.workoutsQuery.data).toEqual([])
        })
    })

    describe('Create Workout Mutation', () => {
        it('creates a new workout', async () => {
            mockedInvoke.mockResolvedValueOnce(mockWorkouts)

            const { result } = renderHook(() => useWorkouts(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.workoutsQuery.isLoading).toBe(false)
            })

            const newWorkout = { name: 'New Workout', notes: 'Test workout' }
            const createdWorkout = { id: 3, ...newWorkout, started_at: '2026-01-19T12:00:00Z' }

            mockedInvoke.mockResolvedValueOnce(createdWorkout)
            mockedInvoke.mockResolvedValueOnce([...mockWorkouts, createdWorkout])

            await act(async () => {
                await result.current.createWorkout.mutateAsync(newWorkout)
            })

            expect(mockedInvoke).toHaveBeenCalledWith('create_workout', { data: newWorkout })
        })
    })

    describe('Update Workout Mutation', () => {
        it('updates an existing workout', async () => {
            mockedInvoke.mockResolvedValueOnce(mockWorkouts)

            const { result } = renderHook(() => useWorkouts(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.workoutsQuery.isLoading).toBe(false)
            })

            const updateData = { notes: 'Updated notes' }
            const updatedWorkout = { ...mockWorkouts[0], ...updateData }

            mockedInvoke.mockResolvedValueOnce(updatedWorkout)
            mockedInvoke.mockResolvedValueOnce(mockWorkouts)

            await act(async () => {
                await result.current.updateWorkout.mutateAsync({ id: 1, data: updateData })
            })

            expect(mockedInvoke).toHaveBeenCalledWith('update_workout', { id: 1, data: updateData })
        })
    })

    describe('Delete Workout Mutation', () => {
        it('deletes a workout', async () => {
            mockedInvoke.mockResolvedValueOnce(mockWorkouts)

            const { result } = renderHook(() => useWorkouts(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.workoutsQuery.isLoading).toBe(false)
            })

            mockedInvoke.mockResolvedValueOnce(true)
            mockedInvoke.mockResolvedValueOnce([mockWorkouts[1]])

            await act(async () => {
                await result.current.deleteWorkout.mutateAsync(1)
            })

            expect(mockedInvoke).toHaveBeenCalledWith('delete_workout', { id: 1 })
        })
    })

    describe('Add Exercise Mutation', () => {
        it('adds an exercise to a workout', async () => {
            mockedInvoke.mockResolvedValueOnce(mockWorkouts)

            const { result } = renderHook(() => useWorkouts(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.workoutsQuery.isLoading).toBe(false)
            })

            const newExercise = {
                workout_id: 1,
                exercise_id: 103,
                sets: 3,
                reps: 15,
                weight: 25,
            }

            mockedInvoke.mockResolvedValueOnce({ id: 3, ...newExercise })
            mockedInvoke.mockResolvedValueOnce(mockWorkouts)

            await act(async () => {
                await result.current.addExercise.mutateAsync(newExercise)
            })

            expect(mockedInvoke).toHaveBeenCalledWith('add_exercise_to_workout', { data: newExercise })
        })
    })

    describe('Update Exercise Mutation', () => {
        it('updates an exercise in a workout', async () => {
            mockedInvoke.mockResolvedValueOnce(mockWorkouts)

            const { result } = renderHook(() => useWorkouts(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.workoutsQuery.isLoading).toBe(false)
            })

            const updateData = { weight: 140, notes: 'Increased weight' }

            mockedInvoke.mockResolvedValueOnce({ ...mockWorkoutExercises[0], ...updateData })
            mockedInvoke.mockResolvedValueOnce(mockWorkouts)

            await act(async () => {
                await result.current.updateExercise.mutateAsync({ id: 1, data: updateData })
            })

            expect(mockedInvoke).toHaveBeenCalledWith('update_workout_exercise', { id: 1, data: updateData })
        })
    })

    describe('Remove Exercise Mutation', () => {
        it('removes an exercise from a workout', async () => {
            mockedInvoke.mockResolvedValueOnce(mockWorkouts)

            const { result } = renderHook(() => useWorkouts(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.workoutsQuery.isLoading).toBe(false)
            })

            mockedInvoke.mockResolvedValueOnce(true)
            mockedInvoke.mockResolvedValueOnce(mockWorkouts)

            await act(async () => {
                await result.current.removeExercise.mutateAsync(1)
            })

            expect(mockedInvoke).toHaveBeenCalledWith('remove_exercise', { id: 1 })
        })
    })

    describe('Check PRs Mutation', () => {
        it('checks and updates personal records after workout', async () => {
            mockedInvoke.mockResolvedValueOnce(mockWorkouts)

            const { result } = renderHook(() => useWorkouts(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.workoutsQuery.isLoading).toBe(false)
            })

            const newPRs = [mockPersonalRecords[0]]

            mockedInvoke.mockResolvedValueOnce(newPRs)
            mockedInvoke.mockResolvedValueOnce(mockPersonalRecords)

            await act(async () => {
                const prs = await result.current.checkPRs.mutateAsync(1)
                expect(prs).toEqual(newPRs)
            })

            expect(mockedInvoke).toHaveBeenCalledWith('check_and_update_prs', { workoutId: 1 })
        })
    })
})

describe('useWorkoutExercises', () => {
    beforeEach(() => {
        vi.clearAllMocks()
    })

    it('fetches exercises for a specific workout', async () => {
        mockedInvoke.mockResolvedValueOnce(mockWorkoutExercises)

        const { result } = renderHook(() => useWorkoutExercises(1), {
            wrapper: createWrapper(),
        })

        await waitFor(() => {
            expect(result.current.isLoading).toBe(false)
        })

        expect(mockedInvoke).toHaveBeenCalledWith('get_workout_exercises', { workoutId: 1 })
        expect(result.current.data).toEqual(mockWorkoutExercises)
    })

    it('is disabled when workoutId is 0', async () => {
        const { result } = renderHook(() => useWorkoutExercises(0), {
            wrapper: createWrapper(),
        })

        // Should not fetch when workoutId is 0
        expect(result.current.isLoading).toBe(false)
        expect(mockedInvoke).not.toHaveBeenCalled()
    })
})

describe('useWorkoutHeatmap', () => {
    beforeEach(() => {
        vi.clearAllMocks()
    })

    it('fetches workout heatmap data for 3 months by default', async () => {
        mockedInvoke.mockResolvedValueOnce(mockHeatmapData)

        const { result } = renderHook(() => useWorkoutHeatmap(), {
            wrapper: createWrapper(),
        })

        await waitFor(() => {
            expect(result.current.isLoading).toBe(false)
        })

        expect(mockedInvoke).toHaveBeenCalledWith('get_workout_heatmap', { months: 3 })
        expect(result.current.data).toEqual(mockHeatmapData)
    })

    it('accepts custom month parameter', async () => {
        mockedInvoke.mockResolvedValueOnce(mockHeatmapData)

        const { result } = renderHook(() => useWorkoutHeatmap(6), {
            wrapper: createWrapper(),
        })

        await waitFor(() => {
            expect(result.current.isLoading).toBe(false)
        })

        expect(mockedInvoke).toHaveBeenCalledWith('get_workout_heatmap', { months: 6 })
    })
})

describe('usePersonalRecords', () => {
    beforeEach(() => {
        vi.clearAllMocks()
    })

    it('fetches personal records', async () => {
        mockedInvoke.mockResolvedValueOnce(mockPersonalRecords)

        const { result } = renderHook(() => usePersonalRecords(), {
            wrapper: createWrapper(),
        })

        await waitFor(() => {
            expect(result.current.isLoading).toBe(false)
        })

        expect(mockedInvoke).toHaveBeenCalledWith('get_personal_records', undefined)
        expect(result.current.data).toEqual(mockPersonalRecords)
    })
})

describe('useWorkoutTemplates', () => {
    beforeEach(() => {
        vi.clearAllMocks()
    })

    describe('Templates Query', () => {
        it('fetches workout templates on mount', async () => {
            mockedInvoke.mockResolvedValueOnce(mockTemplates)

            const { result } = renderHook(() => useWorkoutTemplates(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.templatesQuery.isLoading).toBe(false)
            })

            expect(mockedInvoke).toHaveBeenCalledWith('get_workout_templates', undefined)
            expect(result.current.templatesQuery.data).toEqual(mockTemplates)
        })
    })

    describe('Create Template Mutation', () => {
        it('creates a new workout template', async () => {
            mockedInvoke.mockResolvedValueOnce(mockTemplates)

            const { result } = renderHook(() => useWorkoutTemplates(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.templatesQuery.isLoading).toBe(false)
            })

            const newTemplate = {
                name: 'Leg Day',
                exercises: [{ exercise_id: 201, default_sets: 4, default_reps: 8 }],
            }

            mockedInvoke.mockResolvedValueOnce({ id: 3, ...newTemplate })
            mockedInvoke.mockResolvedValueOnce([...mockTemplates, { id: 3, name: 'Leg Day' }])

            await act(async () => {
                await result.current.createTemplate.mutateAsync(newTemplate)
            })

            expect(mockedInvoke).toHaveBeenCalledWith('create_workout_template', newTemplate)
        })
    })

    describe('Update Template Mutation', () => {
        it('updates an existing template', async () => {
            mockedInvoke.mockResolvedValueOnce(mockTemplates)

            const { result } = renderHook(() => useWorkoutTemplates(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.templatesQuery.isLoading).toBe(false)
            })

            const updateData = {
                id: 1,
                name: 'Push Day Updated',
                exercises: [{ exercise_id: 101, default_sets: 5, default_reps: 8 }],
            }

            mockedInvoke.mockResolvedValueOnce({ ...mockTemplates[0], name: updateData.name })
            mockedInvoke.mockResolvedValueOnce(mockTemplates)

            await act(async () => {
                await result.current.updateTemplate.mutateAsync(updateData)
            })

            expect(mockedInvoke).toHaveBeenCalledWith('update_workout_template', updateData)
        })
    })

    describe('Delete Template Mutation', () => {
        it('deletes a workout template', async () => {
            mockedInvoke.mockResolvedValueOnce(mockTemplates)

            const { result } = renderHook(() => useWorkoutTemplates(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.templatesQuery.isLoading).toBe(false)
            })

            mockedInvoke.mockResolvedValueOnce(true)
            mockedInvoke.mockResolvedValueOnce([mockTemplates[1]])

            await act(async () => {
                await result.current.deleteTemplate.mutateAsync(1)
            })

            expect(mockedInvoke).toHaveBeenCalledWith('delete_workout_template', { id: 1 })
        })
    })
})

describe('useTemplateExercises', () => {
    beforeEach(() => {
        vi.clearAllMocks()
    })

    it('fetches exercises for a specific template', async () => {
        mockedInvoke.mockResolvedValueOnce(mockTemplateExercises)

        const { result } = renderHook(() => useTemplateExercises(1), {
            wrapper: createWrapper(),
        })

        await waitFor(() => {
            expect(result.current.isLoading).toBe(false)
        })

        expect(mockedInvoke).toHaveBeenCalledWith('get_template_exercises', { templateId: 1 })
        expect(result.current.data).toEqual(mockTemplateExercises)
    })

    it('is disabled when templateId is 0', async () => {
        const { result } = renderHook(() => useTemplateExercises(0), {
            wrapper: createWrapper(),
        })

        expect(result.current.isLoading).toBe(false)
        expect(mockedInvoke).not.toHaveBeenCalled()
    })
})
