import { Trash2Icon } from 'lucide-react'
import { useWorkouts } from '@/hooks/useWorkouts'
import { fromNow } from '@/lib/time'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { WorkoutForm } from './workout-form'

export function WorkoutList() {
  const { workoutsQuery, deleteWorkout } = useWorkouts()
  const workouts = workoutsQuery.data ?? []

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Workouts</CardTitle>
        <WorkoutForm />
      </CardHeader>
      <CardContent className="space-y-3">
        {workouts.length === 0 && (
          <p className="text-muted-foreground text-sm">
            No workouts yet. Log one to start tracking!
          </p>
        )}
        {workouts.map((workout) => (
          <div
            key={workout.id}
            className="border-border bg-muted/60 flex items-center justify-between rounded-md border p-3"
          >
            <div>
              <p className="font-medium">Workout</p>
              <p className="text-muted-foreground text-xs">
                Logged {fromNow(workout.logged_at)}
              </p>
            </div>
            <div className="flex items-center gap-1">
              {workout.duration_minutes && (
                <Badge variant="outline">{workout.duration_minutes} min</Badge>
              )}
              <Button
                variant="ghost"
                size="icon-sm"
                onClick={() => deleteWorkout.mutate(workout.id)}
              >
                <Trash2Icon className="h-3 w-3 text-destructive" />
              </Button>
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}
