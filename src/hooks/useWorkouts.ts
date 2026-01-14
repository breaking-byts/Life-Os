import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Workout, WorkoutExercise, WorkoutTemplateExercise } from '@/types'
import { tauri } from '@/lib/tauri'

const WORKOUTS_KEY = ['workouts']
const TEMPLATES_KEY = ['workout-templates']

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

  const updateWorkout = useMutation({
    mutationFn: ({ id, data }: { id: number; data: Partial<Workout> }) =>
      tauri.updateWorkout(id, data),
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

  // Check PRs after workout (returns new PRs if any)
  const checkPRs = useMutation({
    mutationFn: (workoutId: number) => tauri.checkAndUpdatePrs(workoutId),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ['personal-records'] }),
  })

  return {
    workoutsQuery,
    createWorkout,
    updateWorkout,
    deleteWorkout,
    addExercise,
    updateExercise,
    removeExercise,
    checkPRs,
  }
}

// Get exercises for a specific workout
export function useWorkoutExercises(workoutId: number) {
  return useQuery({
    queryKey: ['workout-exercises', workoutId],
    queryFn: () => tauri.getWorkoutExercises(workoutId),
    enabled: workoutId > 0,
  })
}

// Heatmap query
export function useWorkoutHeatmap(months: number = 3) {
  return useQuery({
    queryKey: ['workout-heatmap', months],
    queryFn: () => tauri.getWorkoutHeatmap(months),
  })
}

// Personal records query
export function usePersonalRecords() {
  return useQuery({
    queryKey: ['personal-records'],
    queryFn: tauri.getPersonalRecords,
  })
}

// Templates query
export function useWorkoutTemplates() {
  const queryClient = useQueryClient()

  const templatesQuery = useQuery({
    queryKey: TEMPLATES_KEY,
    queryFn: tauri.getWorkoutTemplates,
  })

  const createTemplate = useMutation({
    mutationFn: ({
      name,
      exercises,
    }: {
      name: string
      exercises: Partial<WorkoutTemplateExercise>[]
    }) => tauri.createWorkoutTemplate(name, exercises),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: TEMPLATES_KEY }),
  })

  const updateTemplate = useMutation({
    mutationFn: ({
      id,
      name,
      exercises,
    }: {
      id: number
      name: string
      exercises: Partial<WorkoutTemplateExercise>[]
    }) => tauri.updateWorkoutTemplate(id, name, exercises),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: TEMPLATES_KEY }),
  })

  const deleteTemplate = useMutation({
    mutationFn: (id: number) => tauri.deleteWorkoutTemplate(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: TEMPLATES_KEY }),
  })

  return {
    templatesQuery,
    createTemplate,
    updateTemplate,
    deleteTemplate,
  }
}

// Get exercises for a specific template
export function useTemplateExercises(templateId: number) {
  return useQuery({
    queryKey: ['template-exercises', templateId],
    queryFn: () => tauri.getTemplateExercises(templateId),
    enabled: templateId > 0,
  })
}
