import {  ActivityCalendar } from 'react-activity-calendar'
import type {Activity} from 'react-activity-calendar';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useWorkoutHeatmap } from '@/hooks/useStats'

export function WorkoutHeatmap() {
  const { data: heatmapData, isLoading } = useWorkoutHeatmap(3)

  // Transform WorkoutHeatmapDay[] to ActivityCalendar format
  const activities: Array<Activity> = (heatmapData ?? []).map((day) => ({
    date: day.date,
    count: day.count,
    level: Math.min(4, day.count) as 0 | 1 | 2 | 3 | 4,
  }))

  // Ensure we have at least some dates for the calendar to render
  if (activities.length === 0) {
    const today = new Date()
    const startDate = new Date(today)
    startDate.setMonth(startDate.getMonth() - 3)

    // Add start and end dates with count 0
    activities.push({
      date: startDate.toISOString().split('T')[0],
      count: 0,
      level: 0,
    })
    activities.push({
      date: today.toISOString().split('T')[0],
      count: 0,
      level: 0,
    })
  }

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Workout Activity</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-32 animate-pulse bg-muted rounded" />
        </CardContent>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Workout Activity</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="overflow-x-auto">
          <ActivityCalendar
            data={activities}
            theme={{
              light: ['#f0f0f0', '#c6e48b', '#7bc96f', '#239a3b', '#196127'],
              dark: ['#161b22', '#0e4429', '#006d32', '#26a641', '#39d353'],
            }}
            colorScheme="dark"
            blockSize={12}
            blockMargin={3}
            fontSize={12}
            labels={{
              totalCount: '{{count}} workouts in the last 3 months',
            }}
          />
        </div>
      </CardContent>
    </Card>
  )
}
