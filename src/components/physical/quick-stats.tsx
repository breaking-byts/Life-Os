import { DumbbellIcon, FlameIcon, TargetIcon, TrophyIcon } from 'lucide-react'
import { Card, CardContent } from '@/components/ui/card'
import {
  usePersonalRecords,
  useStats,
  useStreaks,
  useUserSettings,
} from '@/hooks/useStats'

export function QuickStats() {
  const { data: stats } = useStats()
  const { data: streaks } = useStreaks()
  const { data: prs } = usePersonalRecords()
  const { data: settings } = useUserSettings()

  const workoutsThisWeek = stats?.workouts_week ?? 0
  const workoutTarget = settings?.weekly_workout_target ?? 4
  const workoutStreak = streaks?.workout_streak ?? 0

  // Count PRs from the last 7 days
  const now = new Date()
  const oneWeekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000)
  const newPRsThisWeek =
    prs?.filter((pr) => {
      const achievedAt = new Date(pr.achieved_at)
      return achievedAt >= oneWeekAgo
    }).length ?? 0

  const statCards = [
    {
      label: 'This Week',
      value: `${workoutsThisWeek} / ${workoutTarget}`,
      icon: DumbbellIcon,
      color: 'text-blue-500',
      bgColor: 'bg-blue-500/10',
    },
    {
      label: 'Current Streak',
      value: `${workoutStreak} days`,
      icon: FlameIcon,
      color: 'text-orange-500',
      bgColor: 'bg-orange-500/10',
    },
    {
      label: 'Weekly Target',
      value:
        workoutsThisWeek >= workoutTarget
          ? 'Achieved!'
          : `${workoutTarget - workoutsThisWeek} to go`,
      icon: TargetIcon,
      color:
        workoutsThisWeek >= workoutTarget
          ? 'text-green-500'
          : 'text-yellow-500',
      bgColor:
        workoutsThisWeek >= workoutTarget
          ? 'bg-green-500/10'
          : 'bg-yellow-500/10',
    },
    {
      label: 'New PRs',
      value: `${newPRsThisWeek} this week`,
      icon: TrophyIcon,
      color: 'text-purple-500',
      bgColor: 'bg-purple-500/10',
    },
  ]

  return (
    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
      {statCards.map((stat) => (
        <Card key={stat.label}>
          <CardContent className="flex items-center gap-4 p-4">
            <div className={`rounded-lg p-2.5 ${stat.bgColor}`}>
              <stat.icon className={`h-5 w-5 ${stat.color}`} />
            </div>
            <div>
              <p className="text-muted-foreground text-xs font-medium">
                {stat.label}
              </p>
              <p className="text-lg font-semibold">{stat.value}</p>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  )
}
