import { useState } from 'react'
import { WorkoutForm } from './workout-form'
import { WorkoutCard } from './workout-card'
import type { Workout, WorkoutExercise } from '@/types'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useWorkouts } from '@/hooks/useWorkouts'

export function WorkoutList() {
  const { workoutsQuery, deleteWorkout } = useWorkouts()
  const workouts = workoutsQuery.data ?? []

  // State for edit mode
  const [editingWorkout, setEditingWorkout] = useState<{
    workout: Workout
    exercises: Array<WorkoutExercise>
  } | null>(null)

  const handleEdit = (workout: Workout, exercises: Array<WorkoutExercise>) => {
    setEditingWorkout({ workout, exercises })
  }

  const handleDelete = (id: number) => {
    deleteWorkout.mutate(id)
  }

  const handleFormClose = () => {
    setEditingWorkout(null)
  }

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Workouts</CardTitle>
        <WorkoutForm
          editingWorkout={editingWorkout?.workout}
          editingExercises={editingWorkout?.exercises}
          onClose={handleFormClose}
        />
      </CardHeader>
      <CardContent className="space-y-3">
        {workouts.length === 0 && (
          <p className="text-muted-foreground text-sm">
            No workouts yet. Log one to start tracking!
          </p>
        )}
        {workouts.map((workout) => (
          <WorkoutCard
            key={workout.id}
            workout={workout}
            onEdit={handleEdit}
            onDelete={handleDelete}
          />
        ))}
      </CardContent>
    </Card>
  )
}
