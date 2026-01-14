import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { tauri } from '@/lib/tauri'

export function useStats() {
  return useQuery({
    queryKey: ['stats'],
    queryFn: tauri.getStats,
  })
}

export function useDetailedStats() {
  return useQuery({
    queryKey: ['detailed-stats'],
    queryFn: tauri.getDetailedStats,
  })
}

export function useStreaks() {
  return useQuery({
    queryKey: ['streaks'],
    queryFn: tauri.getStreaks,
  })
}

export function useUserSettings() {
  return useQuery({
    queryKey: ['user-settings'],
    queryFn: tauri.getUserSettings,
  })
}

export function useUpdateUserSettings() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({
      weeklyWorkoutTarget,
      weeklyActiveSkillsTarget,
    }: {
      weeklyWorkoutTarget: number
      weeklyActiveSkillsTarget: number
    }) =>
      tauri.updateUserSettings(weeklyWorkoutTarget, weeklyActiveSkillsTarget),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['user-settings'] })
      queryClient.invalidateQueries({ queryKey: ['detailed-stats'] })
    },
  })
}

export function useWorkoutHeatmap(months: number = 3) {
  return useQuery({
    queryKey: ['workout-heatmap', months],
    queryFn: () => tauri.getWorkoutHeatmap(months),
  })
}

export function usePersonalRecords() {
  return useQuery({
    queryKey: ['personal-records'],
    queryFn: tauri.getPersonalRecords,
  })
}

export function useCheckAndUpdatePrs() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (workoutId: number) => tauri.checkAndUpdatePrs(workoutId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['personal-records'] })
      queryClient.invalidateQueries({ queryKey: ['achievements'] })
    },
  })
}

export function useAchievements() {
  return useQuery({
    queryKey: ['achievements'],
    queryFn: tauri.getAchievements,
  })
}

export function useCheckAchievements() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: tauri.checkAchievements,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['achievements'] })
    },
  })
}
