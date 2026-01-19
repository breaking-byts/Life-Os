/**
 * useIntelligence Hook Tests - TDD Compliant
 *
 * Tests for Agent intelligence hooks:
 * - useIntelligence: recommendations, status, context, feedback
 * - useBigThree: daily goals management
 * - useAgentMaintenance: agent maintenance operations
 *
 * Following Red-Green-Refactor cycle from test_driven_development.md skill
 */

import { renderHook, waitFor, act } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { useIntelligence, useBigThree, useAgentMaintenance } from '../useIntelligence'
import type { ReactNode } from 'react'

// Mock the Tauri invoke function
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}))

const mockedInvoke = vi.mocked(invoke)

// Test fixtures following TDD skill guidance: use real data structures
const mockRecommendations = [
    {
        id: 1,
        action_type: 'study',
        title: 'Review Calculus',
        description: 'Focus on integration techniques',
        confidence: 0.85,
        reasoning: 'You have an exam next week',
        created_at: '2026-01-19T10:00:00Z',
    },
    {
        id: 2,
        action_type: 'workout',
        title: 'Upper body workout',
        description: 'Focus on chest and shoulders',
        confidence: 0.72,
        reasoning: 'Last workout was 3 days ago',
        created_at: '2026-01-19T10:00:00Z',
    },
]

const mockAgentStatus = {
    total_actions: 150,
    total_recommendations: 45,
    acceptance_rate: 0.67,
    average_outcome: 3.8,
    learning_progress: 0.45,
    exploration_rate: 0.1,
    last_maintenance: '2026-01-19T08:00:00Z',
}

const mockRichContext = {
    current_time_block: 'morning',
    day_of_week: 'Monday',
    energy_estimate: 'high',
    pending_assignments: 3,
    upcoming_exams: 1,
    recent_activities: ['studied math', 'completed workout'],
    streak_status: { study: 5, workout: 3 },
}

const mockBigThreeGoals = [
    {
        id: 1,
        title: 'Complete assignment',
        priority: 1,
        completed: false,
        created_at: '2026-01-19T00:00:00Z',
    },
    {
        id: 2,
        title: 'Workout session',
        priority: 2,
        completed: false,
        created_at: '2026-01-19T00:00:00Z',
    },
    {
        id: 3,
        title: 'Read 30 pages',
        priority: 3,
        completed: true,
        satisfaction: 4,
        created_at: '2026-01-19T00:00:00Z',
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
            <QueryClientProvider client= { queryClient } >
            { children }
            </QueryClientProvider>
        )
    }
}

describe('useIntelligence', () => {
    beforeEach(() => {
        vi.clearAllMocks()
    })

    afterEach(() => {
        vi.resetAllMocks()
    })

    describe('Recommendations Query', () => {
        it('fetches agent recommendations on mount', async () => {
            mockedInvoke.mockResolvedValueOnce(mockRecommendations)
            mockedInvoke.mockResolvedValueOnce(mockAgentStatus)
            mockedInvoke.mockResolvedValueOnce(mockRichContext)

            const { result } = renderHook(() => useIntelligence(), {
                wrapper: createWrapper(),
            })

            expect(result.current.isLoadingRecommendations).toBe(true)

            await waitFor(() => {
                expect(result.current.isLoadingRecommendations).toBe(false)
            })

            expect(mockedInvoke).toHaveBeenCalledWith('get_agent_recommendations', { count: 3 })
            expect(result.current.recommendations).toEqual(mockRecommendations)
        })

        it('returns empty array when no recommendations available', async () => {
            mockedInvoke.mockResolvedValueOnce([])
            mockedInvoke.mockResolvedValueOnce(mockAgentStatus)
            mockedInvoke.mockResolvedValueOnce(mockRichContext)

            const { result } = renderHook(() => useIntelligence(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.isLoadingRecommendations).toBe(false)
            })

            expect(result.current.recommendations).toEqual([])
        })

        it('handles recommendation fetch error gracefully', async () => {
            const error = new Error('Agent not ready')
            mockedInvoke.mockRejectedValueOnce(error)
            mockedInvoke.mockResolvedValueOnce(mockAgentStatus)
            mockedInvoke.mockResolvedValueOnce(mockRichContext)

            const { result } = renderHook(() => useIntelligence(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.isLoadingRecommendations).toBe(false)
            })

            expect(result.current.recommendationsError).toBeTruthy()
            expect(result.current.recommendations).toEqual([])
        })
    })

    describe('Agent Status Query', () => {
        it('fetches agent status on mount', async () => {
            mockedInvoke.mockResolvedValueOnce(mockRecommendations)
            mockedInvoke.mockResolvedValueOnce(mockAgentStatus)
            mockedInvoke.mockResolvedValueOnce(mockRichContext)

            const { result } = renderHook(() => useIntelligence(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.isLoadingStatus).toBe(false)
            })

            expect(mockedInvoke).toHaveBeenCalledWith('get_agent_status')
            expect(result.current.status).toEqual(mockAgentStatus)
        })
    })

    describe('Rich Context Query', () => {
        it('fetches rich context on mount', async () => {
            mockedInvoke.mockResolvedValueOnce(mockRecommendations)
            mockedInvoke.mockResolvedValueOnce(mockAgentStatus)
            mockedInvoke.mockResolvedValueOnce(mockRichContext)

            const { result } = renderHook(() => useIntelligence(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.isLoadingContext).toBe(false)
            })

            expect(mockedInvoke).toHaveBeenCalledWith('get_rich_context')
            expect(result.current.context).toEqual(mockRichContext)
        })
    })

    describe('Record Feedback Mutation', () => {
        it('records positive feedback on recommendation', async () => {
            mockedInvoke.mockResolvedValueOnce(mockRecommendations)
            mockedInvoke.mockResolvedValueOnce(mockAgentStatus)
            mockedInvoke.mockResolvedValueOnce(mockRichContext)

            const { result } = renderHook(() => useIntelligence(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.isLoadingRecommendations).toBe(false)
            })

            // Setup mock for feedback recording
            mockedInvoke.mockResolvedValueOnce(undefined)
            // Setup mock for refetch after mutation
            mockedInvoke.mockResolvedValueOnce([])

            await act(async () => {
                await result.current.recordFeedback.mutateAsync({
                    recommendationId: 1,
                    accepted: true,
                    feedbackScore: 5,
                    outcomeScore: 4,
                })
            })

            expect(mockedInvoke).toHaveBeenCalledWith('record_recommendation_feedback', {
                recommendationId: 1,
                accepted: true,
                alternativeChosen: undefined,
                feedbackScore: 5,
                outcomeScore: 4,
            })
        })

        it('records negative feedback with alternative chosen', async () => {
            mockedInvoke.mockResolvedValueOnce(mockRecommendations)
            mockedInvoke.mockResolvedValueOnce(mockAgentStatus)
            mockedInvoke.mockResolvedValueOnce(mockRichContext)

            const { result } = renderHook(() => useIntelligence(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.isLoadingRecommendations).toBe(false)
            })

            mockedInvoke.mockResolvedValueOnce(undefined)
            mockedInvoke.mockResolvedValueOnce([])

            await act(async () => {
                await result.current.recordFeedback.mutateAsync({
                    recommendationId: 1,
                    accepted: false,
                    alternativeChosen: 'rest',
                    feedbackScore: 2,
                })
            })

            expect(mockedInvoke).toHaveBeenCalledWith('record_recommendation_feedback', {
                recommendationId: 1,
                accepted: false,
                alternativeChosen: 'rest',
                feedbackScore: 2,
                outcomeScore: undefined,
            })
        })
    })

    describe('Record Action Mutation', () => {
        it('records completed action with metadata', async () => {
            mockedInvoke.mockResolvedValueOnce(mockRecommendations)
            mockedInvoke.mockResolvedValueOnce(mockAgentStatus)
            mockedInvoke.mockResolvedValueOnce(mockRichContext)

            const { result } = renderHook(() => useIntelligence(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.isLoadingRecommendations).toBe(false)
            })

            mockedInvoke.mockResolvedValueOnce(1)
            mockedInvoke.mockResolvedValueOnce([])
            mockedInvoke.mockResolvedValueOnce(mockAgentStatus)

            await act(async () => {
                await result.current.recordAction.mutateAsync({
                    actionType: 'workout',
                    description: 'Completed upper body workout',
                    outcomeScore: 4,
                    metadata: { duration_minutes: 45, exercises: 6 },
                })
            })

            expect(mockedInvoke).toHaveBeenCalledWith('record_action_completed', {
                actionType: 'workout',
                description: 'Completed upper body workout',
                outcomeScore: 4,
                metadata: { duration_minutes: 45, exercises: 6 },
            })
        })
    })
})

describe('useBigThree', () => {
    beforeEach(() => {
        vi.clearAllMocks()
    })

    describe('Goals Query', () => {
        it('fetches Big Three goals on mount', async () => {
            mockedInvoke.mockResolvedValueOnce(mockBigThreeGoals)

            const { result } = renderHook(() => useBigThree(), {
                wrapper: createWrapper(),
            })

            expect(result.current.isLoading).toBe(true)

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false)
            })

            expect(mockedInvoke).toHaveBeenCalledWith('get_big_three')
            expect(result.current.goals).toEqual(mockBigThreeGoals)
        })

        it('returns empty array when no goals set', async () => {
            mockedInvoke.mockResolvedValueOnce([])

            const { result } = renderHook(() => useBigThree(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false)
            })

            expect(result.current.goals).toEqual([])
        })
    })

    describe('Set Goals Mutation', () => {
        it('sets new Big Three goals', async () => {
            mockedInvoke.mockResolvedValueOnce([])

            const { result } = renderHook(() => useBigThree(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false)
            })

            const newGoals = [
                { title: 'Goal 1', priority: 1 },
                { title: 'Goal 2', priority: 2 },
                { title: 'Goal 3', priority: 3 },
            ]

            mockedInvoke.mockResolvedValueOnce(undefined)
            mockedInvoke.mockResolvedValueOnce(mockBigThreeGoals)

            await act(async () => {
                await result.current.setGoals.mutateAsync(newGoals)
            })

            expect(mockedInvoke).toHaveBeenCalledWith('set_big_three', { goals: newGoals })
        })
    })

    describe('Complete Goal Mutation', () => {
        it('completes a goal with satisfaction score', async () => {
            mockedInvoke.mockResolvedValueOnce(mockBigThreeGoals)

            const { result } = renderHook(() => useBigThree(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false)
            })

            mockedInvoke.mockResolvedValueOnce(undefined)
            mockedInvoke.mockResolvedValueOnce(mockBigThreeGoals)
            mockedInvoke.mockResolvedValueOnce([]) // For recommendations invalidation

            await act(async () => {
                await result.current.completeGoal.mutateAsync({
                    goalId: 1,
                    satisfaction: 5,
                })
            })

            expect(mockedInvoke).toHaveBeenCalledWith('complete_big_three', {
                goalId: 1,
                satisfaction: 5,
            })
        })

        it('completes a goal without satisfaction score', async () => {
            mockedInvoke.mockResolvedValueOnce(mockBigThreeGoals)

            const { result } = renderHook(() => useBigThree(), {
                wrapper: createWrapper(),
            })

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false)
            })

            mockedInvoke.mockResolvedValueOnce(undefined)
            mockedInvoke.mockResolvedValueOnce(mockBigThreeGoals)
            mockedInvoke.mockResolvedValueOnce([])

            await act(async () => {
                await result.current.completeGoal.mutateAsync({
                    goalId: 2,
                })
            })

            expect(mockedInvoke).toHaveBeenCalledWith('complete_big_three', {
                goalId: 2,
                satisfaction: undefined,
            })
        })
    })
})

describe('useAgentMaintenance', () => {
    beforeEach(() => {
        vi.clearAllMocks()
    })

    describe('Run Maintenance Mutation', () => {
        it('triggers agent maintenance operation', async () => {
            const { result } = renderHook(() => useAgentMaintenance(), {
                wrapper: createWrapper(),
            })

            mockedInvoke.mockResolvedValueOnce(undefined)
            mockedInvoke.mockResolvedValueOnce(mockAgentStatus)

            await act(async () => {
                await result.current.runMaintenance.mutateAsync()
            })

            expect(mockedInvoke).toHaveBeenCalledWith('run_agent_maintenance')
        })
    })

    describe('Set Exploration Rate Mutation', () => {
        it('sets exploration rate within valid range', async () => {
            const { result } = renderHook(() => useAgentMaintenance(), {
                wrapper: createWrapper(),
            })

            mockedInvoke.mockResolvedValueOnce(undefined)
            mockedInvoke.mockResolvedValueOnce(mockAgentStatus)

            await act(async () => {
                await result.current.setExplorationRate.mutateAsync(0.15)
            })

            expect(mockedInvoke).toHaveBeenCalledWith('set_exploration_rate', { rate: 0.15 })
        })
    })
})
