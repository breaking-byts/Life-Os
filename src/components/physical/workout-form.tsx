import { useEffect, useState } from 'react'
import { PlusIcon, SaveIcon, Trash2Icon } from 'lucide-react'
import { toast } from 'sonner'
import { ExerciseSearch } from './exercise-search'
import { TemplateSelector } from './template-selector'
import type { Exercise, Workout, WorkoutExercise } from '@/types'
import { useWorkoutTemplates, useWorkouts } from '@/hooks/useWorkouts'
import { useCheckAndUpdatePrs } from '@/hooks/useStats'
import { formatPRValue } from '@/lib/workout-utils'
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

interface ExerciseEntry {
  exercise_name: string
  exercise_id?: number
  sets?: string
  reps?: string
  weight?: string
}

interface WorkoutFormProps {
  editingWorkout?: Workout
  editingExercises?: Array<WorkoutExercise>
  onClose?: () => void
}

export function WorkoutForm({
  editingWorkout,
  editingExercises,
  onClose,
}: WorkoutFormProps) {
  const [open, setOpen] = useState(false)
  const { createWorkout, updateWorkout, addExercise, removeExercise } =
    useWorkouts()
  const { createTemplate } = useWorkoutTemplates()
  const checkPRs = useCheckAndUpdatePrs()

  const [name, setName] = useState('')
  const [duration, setDuration] = useState('')
  const [notes, setNotes] = useState('')
  const [exercises, setExercises] = useState<Array<ExerciseEntry>>([])
  const [saveAsTemplate, setSaveAsTemplate] = useState(false)

  const isEditing = !!editingWorkout

  // Populate form when editing
  useEffect(() => {
    if (editingWorkout) {
      setName(editingWorkout.name || '')
      setDuration(editingWorkout.duration_minutes?.toString() || '')
      setNotes(editingWorkout.notes || '')
      if (editingExercises) {
        setExercises(
          editingExercises.map((ex) => ({
            exercise_name: ex.exercise_name,
            exercise_id: ex.exercise_id ?? undefined,
            sets: ex.sets?.toString() ?? '',
            reps: ex.reps?.toString() ?? '',
            weight: ex.weight?.toString() ?? '',
          })),
        )
      }
      setOpen(true)
    }
  }, [editingWorkout, editingExercises])

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

  const handleLoadTemplate = (templateExercises: Array<ExerciseEntry>) => {
    setExercises([...exercises, ...templateExercises])
  }

  const handleRemoveExercise = (index: number) => {
    setExercises(exercises.filter((_, i) => i !== index))
  }

  const updateExerciseField = (
    index: number,
    field: keyof ExerciseEntry,
    value: string,
  ) => {
    setExercises(
      exercises.map((ex, i) => (i === index ? { ...ex, [field]: value } : ex)),
    )
  }

  const resetForm = () => {
    setName('')
    setDuration('')
    setNotes('')
    setExercises([])
    setSaveAsTemplate(false)
  }

  const handleClose = () => {
    resetForm()
    setOpen(false)
    onClose?.()
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    try {
      let workoutId: number

      if (editingWorkout) {
        // Update existing workout
        await updateWorkout.mutateAsync({
          id: editingWorkout.id,
          data: {
            name: name || undefined,
            duration_minutes: duration ? parseInt(duration) : undefined,
            notes: notes || undefined,
          },
        })
        workoutId = editingWorkout.id

        // Remove old exercises and add new ones
        if (editingExercises) {
          await Promise.all(
            editingExercises.map((ex) => removeExercise.mutateAsync(ex.id)),
          )
        }
      } else {
        // Create new workout
        const workout = await createWorkout.mutateAsync({
          name: name || undefined,
          duration_minutes: duration ? parseInt(duration) : undefined,
          notes: notes || undefined,
        })
        workoutId = workout.id
      }

      // Add exercises
      await Promise.all(
        exercises.map((ex) =>
          addExercise.mutateAsync({
            workout_id: workoutId,
            exercise_id: ex.exercise_id,
            exercise_name: ex.exercise_name,
            sets: ex.sets ? parseInt(ex.sets) : undefined,
            reps: ex.reps ? parseInt(ex.reps) : undefined,
            weight: ex.weight ? parseFloat(ex.weight) : undefined,
          }),
        ),
      )

      // Check for new PRs
      const newPRs = await checkPRs.mutateAsync(workoutId)
      if (newPRs.length > 0) {
        newPRs.forEach((pr) => {
          toast.success(
            `New PR! ${pr.exercise_name}: ${formatPRValue(pr.pr_type, pr.value)}`,
            { icon: '🏆' },
          )
        })
      }

      // Save as template if requested and name is provided
      if (saveAsTemplate && name) {
        await createTemplate.mutateAsync({
          name,
          exercises: exercises.map((ex, idx) => ({
            exercise_name: ex.exercise_name,
            exercise_id: ex.exercise_id,
            default_sets: ex.sets ? parseInt(ex.sets) : undefined,
            default_reps: ex.reps ? parseInt(ex.reps) : undefined,
            default_weight: ex.weight ? parseFloat(ex.weight) : undefined,
            order_index: idx,
          })),
        })
        toast.success(`Template "${name}" saved!`)
      }

      handleClose()
    } catch (error) {
      console.error('Failed to save workout:', error)
      toast.error('Failed to save workout')
    }
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(isOpen) => {
        if (!isOpen) handleClose()
        else setOpen(true)
      }}
    >
      <DialogTrigger asChild>
        <Button size="sm">
          <PlusIcon className="mr-1 h-4 w-4" />
          Log Workout
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-lg">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>
              {isEditing ? 'Edit Workout' : 'Log Workout'}
            </DialogTitle>
            <DialogDescription>
              {isEditing
                ? 'Update your workout details.'
                : 'Record a workout session with exercises.'}
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-4 py-4 max-h-[60vh] overflow-y-auto">
            {/* Workout Name (optional) */}
            <div className="grid gap-2">
              <Label htmlFor="name">Workout Name (optional)</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="e.g., Push Day, Leg Day, Upper Body"
              />
              <p className="text-xs text-muted-foreground">
                Give your workout a name to save it as a reusable template.
              </p>
            </div>

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
              <div className="flex items-center justify-between">
                <Label>Exercises</Label>
                <TemplateSelector onSelectTemplate={handleLoadTemplate} />
              </div>
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
                            updateExerciseField(index, 'sets', e.target.value)
                          }
                        />
                        <Input
                          type="number"
                          placeholder="Reps"
                          className="w-16 h-7 text-xs"
                          value={ex.reps}
                          onChange={(e) =>
                            updateExerciseField(index, 'reps', e.target.value)
                          }
                        />
                        <Input
                          type="number"
                          placeholder="Weight"
                          className="w-20 h-7 text-xs"
                          value={ex.weight}
                          onChange={(e) =>
                            updateExerciseField(index, 'weight', e.target.value)
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

            {/* Save as template option */}
            {name && !isEditing && (
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="saveAsTemplate"
                  checked={saveAsTemplate}
                  onChange={(e) => setSaveAsTemplate(e.target.checked)}
                  className="h-4 w-4 rounded border-gray-300"
                />
                <Label
                  htmlFor="saveAsTemplate"
                  className="text-sm font-normal cursor-pointer"
                >
                  Save as template for future workouts
                </Label>
              </div>
            )}
          </div>

          <DialogFooter className="gap-2">
            <Button type="button" variant="outline" onClick={handleClose}>
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={createWorkout.isPending || updateWorkout.isPending}
            >
              {createWorkout.isPending || updateWorkout.isPending ? (
                'Saving...'
              ) : isEditing ? (
                'Update Workout'
              ) : (
                <>
                  <SaveIcon className="mr-1 h-4 w-4" />
                  Log Workout
                </>
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
