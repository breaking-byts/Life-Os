import { useState } from 'react'
import { PlusIcon, Trash2Icon } from 'lucide-react'
import { useWorkouts } from '@/hooks/useWorkouts'
import type { Exercise } from '@/types'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { ExerciseSearch } from './exercise-search'

interface ExerciseEntry {
  exercise_name: string
  exercise_id?: number
  sets?: string
  reps?: string
  weight?: string
}

export function WorkoutForm() {
  const [open, setOpen] = useState(false)
  const { createWorkout, addExercise } = useWorkouts()

  const [duration, setDuration] = useState('')
  const [notes, setNotes] = useState('')
  const [exercises, setExercises] = useState<ExerciseEntry[]>([])

  const handleAddExercise = (exercise: Exercise) => {
    setExercises([
      ...exercises,
      {
        exercise_name: exercise.name,
        exercise_id: exercise.id,
        sets: '',
        reps: '',
        weight: '',
      },
    ])
  }

  const handleRemoveExercise = (index: number) => {
    setExercises(exercises.filter((_, i) => i !== index))
  }

  const updateExercise = (
    index: number,
    field: keyof ExerciseEntry,
    value: string,
  ) => {
    setExercises(
      exercises.map((ex, i) => (i === index ? { ...ex, [field]: value } : ex)),
    )
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    const workout = await createWorkout.mutateAsync({
      duration_minutes: duration ? parseInt(duration) : undefined,
      notes: notes || undefined,
    })

    await Promise.all(
      exercises.map((ex) =>
        addExercise.mutateAsync({
          workout_id: workout.id,
          exercise_id: ex.exercise_id,
          exercise_name: ex.exercise_name,
          sets: ex.sets ? parseInt(ex.sets) : undefined,
          reps: ex.reps ? parseInt(ex.reps) : undefined,
          weight: ex.weight ? parseFloat(ex.weight) : undefined,
        }),
      ),
    )

    // Reset form
    setDuration('')
    setNotes('')
    setExercises([])
    setOpen(false)
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="sm">
          <PlusIcon className="mr-1 h-4 w-4" />
          Log Workout
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-lg">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>Log Workout</DialogTitle>
            <DialogDescription>
              Record a workout session with exercises.
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-4 py-4 max-h-[60vh] overflow-y-auto">
            <div className="grid gap-2">
              <Label htmlFor="duration">Duration (min)</Label>
              <Input
                id="duration"
                type="number"
                min="1"
                max="300"
                value={duration}
                onChange={(e) => setDuration(e.target.value)}
                placeholder="45"
              />
            </div>

            <div className="grid gap-2">
              <Label>Add Exercises</Label>
              <ExerciseSearch onSelect={handleAddExercise} />
            </div>

            {exercises.length > 0 && (
              <div className="space-y-2">
                {exercises.map((ex, index) => (
                  <div
                    key={index}
                    className="flex items-center gap-2 p-2 border rounded-md bg-muted/50"
                  >
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium truncate">
                        {ex.exercise_name}
                      </p>
                      <div className="flex gap-2 mt-1">
                        <Input
                          type="number"
                          placeholder="Sets"
                          className="w-16 h-7 text-xs"
                          value={ex.sets}
                          onChange={(e) =>
                            updateExercise(index, 'sets', e.target.value)
                          }
                        />
                        <Input
                          type="number"
                          placeholder="Reps"
                          className="w-16 h-7 text-xs"
                          value={ex.reps}
                          onChange={(e) =>
                            updateExercise(index, 'reps', e.target.value)
                          }
                        />
                        <Input
                          type="number"
                          placeholder="Weight"
                          className="w-20 h-7 text-xs"
                          value={ex.weight}
                          onChange={(e) =>
                            updateExercise(index, 'weight', e.target.value)
                          }
                        />
                      </div>
                    </div>
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon-sm"
                      onClick={() => handleRemoveExercise(index)}
                    >
                      <Trash2Icon className="h-3 w-3 text-destructive" />
                    </Button>
                  </div>
                ))}
              </div>
            )}

            <div className="grid gap-2">
              <Label htmlFor="notes">Notes</Label>
              <Textarea
                id="notes"
                value={notes}
                onChange={(e) => setNotes(e.target.value)}
                placeholder="How did the workout go?"
                rows={2}
              />
            </div>
          </div>

          <DialogFooter>
            <Button type="submit" disabled={createWorkout.isPending}>
              {createWorkout.isPending ? 'Saving...' : 'Log Workout'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
