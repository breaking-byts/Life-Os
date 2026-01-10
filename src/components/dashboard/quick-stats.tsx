import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { useStats } from '@/hooks/useStats'

export function QuickStats() {
  const { data } = useStats()

  const stats = [
    { label: 'Study hours', value: data?.study_hours_week ?? 0, target: 12 },
    { label: 'Practice hours', value: data?.practice_hours_week ?? 0, target: 8 },
    { label: 'Workouts', value: data?.workouts_week ?? 0, target: 3 },
    { label: 'Active skills', value: data?.active_streaks ?? 0, target: 5 },
  ]

  return (
    <div className="grid gap-4 md:grid-cols-4">
      {stats.map((stat) => {
        const percent = Math.min(100, Math.round((stat.value / stat.target) * 100))
        return (
          <Card key={stat.label}>
            <CardHeader>
              <CardTitle className="text-base">{stat.label}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">{stat.value}</span>
                <span>Target {stat.target}</span>
              </div>
              <Progress value={percent} />
            </CardContent>
          </Card>
        )
      })}
    </div>
  )
}
