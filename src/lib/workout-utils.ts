import type { WorkoutExercise } from '@/types'

// Generate a label from exercises based on muscle groups
export function generateWorkoutLabel(
  exercises: Array<WorkoutExercise>,
): string {
  if (exercises.length === 0) return 'Workout'

  // Extract unique exercise names and truncate
  const names = exercises.slice(0, 3).map((e) => e.exercise_name)
  if (exercises.length > 3) {
    return `${names.join(', ')} +${exercises.length - 3} more`
  }
  return names.join(', ')
}

// Calculate total volume (sets x reps x weight)
export function calculateTotalVolume(
  exercises: Array<WorkoutExercise>,
): number {
  return exercises.reduce((sum, ex) => {
    const sets = ex.sets || 0
    const reps = ex.reps || 0
    const weight = ex.weight || 0
    return sum + sets * reps * weight
  }, 0)
}

// Format volume for display (e.g., "12,500 kgs")
export function formatVolume(volume: number): string {
  if (volume === 0) return ''
  return `${volume.toLocaleString()} kgs`
}

// Format PR value based on type
export function formatPRValue(prType: string, value: number): string {
  switch (prType) {
    case 'weight':
      return `${value} kgs`
    case 'volume':
      return `${value.toLocaleString()} kgs`
    case 'reps':
      return `${value} reps`
    default:
      return `${value}`
  }
}

// Get total sets in a workout
export function getTotalSets(exercises: Array<WorkoutExercise>): number {
  return exercises.reduce((sum, ex) => sum + (ex.sets || 0), 0)
}

// Get total reps in a workout
export function getTotalReps(exercises: Array<WorkoutExercise>): number {
  return exercises.reduce((sum, ex) => {
    const sets = ex.sets || 0
    const reps = ex.reps || 0
    return sum + sets * reps
  }, 0)
}
