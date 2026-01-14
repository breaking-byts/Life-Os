import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { BigThreeInput } from '@/types'
import { tauri } from '@/lib/tauri'

const RECOMMENDATIONS_KEY = ['agent-recommendations']
const BIG_THREE_KEY = ['big-three']
const AGENT_STATUS_KEY = ['agent-status']
const RICH_CONTEXT_KEY = ['rich-context']

export function useIntelligence() {
  const queryClient = useQueryClient()

  // Get agent recommendations
  const recommendationsQuery = useQuery({
    queryKey: RECOMMENDATIONS_KEY,
    queryFn: () => tauri.getAgentRecommendations(3),
    staleTime: 2 * 60 * 1000, // 2 minutes
    retry: false, // Don't retry if the agent isn't ready
  })

  // Get agent status
  const statusQuery = useQuery({
    queryKey: AGENT_STATUS_KEY,
    queryFn: tauri.getAgentStatus,
    staleTime: 5 * 60 * 1000, // 5 minutes
    retry: false,
  })

  // Get rich context
  const contextQuery = useQuery({
    queryKey: RICH_CONTEXT_KEY,
    queryFn: tauri.getRichContext,
    staleTime: 1 * 60 * 1000, // 1 minute
    retry: false,
  })

  // Record feedback on a recommendation
  const recordFeedback = useMutation({
    mutationFn: ({
      recommendationId,
      accepted,
      alternativeChosen,
      feedbackScore,
      outcomeScore,
    }: {
      recommendationId: number
      accepted: boolean
      alternativeChosen?: string
      feedbackScore?: number
      outcomeScore?: number
    }) =>
      tauri.recordRecommendationFeedback(
        recommendationId,
        accepted,
        alternativeChosen,
        feedbackScore,
        outcomeScore,
      ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: RECOMMENDATIONS_KEY })
    },
  })

  // Record a completed action
  const recordAction = useMutation({
    mutationFn: ({
      actionType,
      description,
      outcomeScore,
      metadata,
    }: {
      actionType: string
      description: string
      outcomeScore: number
      metadata?: Record<string, unknown>
    }) =>
      tauri.recordActionCompleted(
        actionType,
        description,
        outcomeScore,
        metadata,
      ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: RECOMMENDATIONS_KEY })
      queryClient.invalidateQueries({ queryKey: AGENT_STATUS_KEY })
    },
  })

  return {
    recommendations: recommendationsQuery.data ?? [],
    isLoadingRecommendations: recommendationsQuery.isLoading,
    recommendationsError: recommendationsQuery.error,
    refetchRecommendations: recommendationsQuery.refetch,
    isRefetchingRecommendations: recommendationsQuery.isRefetching,

    status: statusQuery.data,
    isLoadingStatus: statusQuery.isLoading,

    context: contextQuery.data,
    isLoadingContext: contextQuery.isLoading,

    recordFeedback,
    recordAction,
  }
}

export function useBigThree() {
  const queryClient = useQueryClient()

  // Get today's Big 3 goals
  const goalsQuery = useQuery({
    queryKey: BIG_THREE_KEY,
    queryFn: tauri.getBigThree,
    staleTime: 30 * 1000, // 30 seconds
  })

  // Set Big 3 goals
  const setGoals = useMutation({
    mutationFn: (goals: BigThreeInput[]) => tauri.setBigThree(goals),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: BIG_THREE_KEY })
    },
  })

  // Complete a goal
  const completeGoal = useMutation({
    mutationFn: ({
      goalId,
      satisfaction,
    }: {
      goalId: number
      satisfaction?: number
    }) => tauri.completeBigThree(goalId, satisfaction),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: BIG_THREE_KEY })
      queryClient.invalidateQueries({ queryKey: RECOMMENDATIONS_KEY })
    },
  })

  return {
    goals: goalsQuery.data ?? [],
    isLoading: goalsQuery.isLoading,
    refetch: goalsQuery.refetch,

    setGoals,
    completeGoal,
  }
}

export function useAgentMaintenance() {
  const queryClient = useQueryClient()

  const runMaintenance = useMutation({
    mutationFn: tauri.runAgentMaintenance,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: AGENT_STATUS_KEY })
    },
  })

  const setExplorationRate = useMutation({
    mutationFn: (rate: number) => tauri.setExplorationRate(rate),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: AGENT_STATUS_KEY })
    },
  })

  return {
    runMaintenance,
    setExplorationRate,
  }
}
