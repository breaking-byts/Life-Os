import { useState } from 'react'
import {
  ChevronDownIcon,
  ChevronUpIcon,
  DumbbellIcon,
  PencilIcon,
  Trash2Icon,
} from 'lucide-react'
import type { Workout, WorkoutExercise } from '@/types'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { fromNow } from '@/lib/time'
import {
  calculateTotalVolume,
  formatVolume,
  generateWorkoutLabel,
} from '@/lib/workout-utils'
import { useWorkoutExercises } from '@/hooks/useWorkouts'

interface WorkoutCardProps {
  workout: Workout
  onEdit?: (workout: Workout, exercises: Array<WorkoutExercise>) => void
  onDelete?: (id: number) => void
}

export function WorkoutCard({ workout, onEdit, onDelete }: WorkoutCardProps) {
  const [isExpanded, setIsExpanded] = useState(false)
  const { data: exercises = [], isLoading } = useWorkoutExercises(workout.id)

  const workoutLabel = workout.name || generateWorkoutLabel(exercises)
  const totalVolume = calculateTotalVolume(exercises)

  return (
    <div className="rounded-lg border border-border bg-card overflow-hidden">
      {/* Collapsed Header */}
      <div
        className="flex items-center justify-between p-4 cursor-pointer hover:bg-muted/50 transition-colors"
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
            <DumbbellIcon className="h-5 w-5 text-primary" />
          </div>
          <div>
            <p className="font-medium">
              {isLoading ? 'Loading...' : workoutLabel}
            </p>
            <p className="text-xs text-muted-foreground">
              {fromNow(workout.logged_at)}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {workout.duration_minutes && (
            <Badge variant="outline">{workout.duration_minutes} min</Badge>
          )}
          {exercises.length > 0 && (
            <Badge variant="secondary">{exercises.length} exercises</Badge>
          )}
          {totalVolume > 0 && (
            <Badge variant="outline" className="hidden sm:flex">
              {formatVolume(totalVolume)}
            </Badge>
          )}
          <Button
            variant="ghost"
            size="icon-sm"
            onClick={(e) => {
              e.stopPropagation()
              setIsExpanded(!isExpanded)
            }}
          >
            {isExpanded ? (
              <ChevronUpIcon className="h-4 w-4" />
            ) : (
              <ChevronDownIcon className="h-4 w-4" />
            )}
          </Button>
        </div>
      </div>

      {/* Expanded Content */}
      {isExpanded && (
        <div className="border-t border-border p-4 bg-muted/30">
          {/* Exercises List */}
          {exercises.length > 0 ? (
            <div className="space-y-2 mb-4">
              {exercises.map((exercise) => (
                <div
                  key={exercise.id}
                  className="flex items-center justify-between text-sm p-2 rounded-md bg-background"
                >
                  <span className="font-medium">{exercise.exercise_name}</span>
                  <div className="flex items-center gap-2 text-muted-foreground">
                    {exercise.sets && exercise.reps && (
                      <span>
                        {exercise.sets} x {exercise.reps}
                      </span>
                    )}
                    {exercise.weight && (
                      <span className="text-foreground font-medium">
                        @ {exercise.weight} kgs
                      </span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground mb-4">
              No exercises logged for this workout.
            </p>
          )}

          {/* Notes */}
          {workout.notes && (
            <div className="mb-4">
              <p className="text-xs text-muted-foreground mb-1">Notes</p>
              <p className="text-sm">{workout.notes}</p>
            </div>
          )}

          {/* Actions */}
          <div className="flex items-center gap-2 justify-end">
            {onEdit && (
              <Button
                variant="outline"
                size="sm"
                onClick={() => onEdit(workout, exercises)}
              >
                <PencilIcon className="h-3 w-3 mr-1" />
                Edit
              </Button>
            )}
            {onDelete && (
              <Button
                variant="outline"
                size="sm"
                onClick={() => onDelete(workout.id)}
              >
                <Trash2Icon className="h-3 w-3 mr-1 text-destructive" />
                Delete
              </Button>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
