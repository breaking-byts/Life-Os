import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Workout, WorkoutExercise } from '@/types'
import { tauri } from '@/lib/tauri'

const WORKOUTS_KEY = ['workouts']

export function useWorkouts() {
  const queryClient = useQueryClient()

  const workoutsQuery = useQuery({
    queryKey: WORKOUTS_KEY,
    queryFn: tauri.getWorkouts,
  })

  const createWorkout = useMutation({
    mutationFn: (data: Partial<Workout>) => tauri.createWorkout(data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: WORKOUTS_KEY }),
  })

  const deleteWorkout = useMutation({
    mutationFn: (id: number) => tauri.deleteWorkout(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: WORKOUTS_KEY }),
  })

  const addExercise = useMutation({
    mutationFn: (data: Partial<WorkoutExercise>) =>
      tauri.addExerciseToWorkout(data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: WORKOUTS_KEY }),
  })

  const updateExercise = useMutation({
    mutationFn: ({
      id,
      data,
    }: {
      id: number
      data: Partial<WorkoutExercise>
    }) => tauri.updateWorkoutExercise(id, data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: WORKOUTS_KEY }),
  })

  const removeExercise = useMutation({
    mutationFn: (id: number) => tauri.removeExercise(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: WORKOUTS_KEY }),
  })

  return {
    workoutsQuery,
    createWorkout,
    deleteWorkout,
    addExercise,
    updateExercise,
    removeExercise,
  }
}
