import { BookOpen, Dumbbell, Flame, Target } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import {
  Popover,
  PopoverContent,
  PopoverDescription,
  PopoverHeader,
  PopoverTitle,
  PopoverTrigger,
} from '@/components/ui/popover'
import { useDetailedStats } from '@/hooks/useStats'
import { cn } from '@/lib/utils'

export function QuickStats() {
  const { data, isLoading } = useDetailedStats()

  if (isLoading) {
    return (
      <div className="grid gap-4 md:grid-cols-4">
        {[1, 2, 3, 4].map((i) => (
          <Card key={i} className="animate-pulse">
            <CardHeader>
              <div className="bg-muted h-4 w-24 rounded" />
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="bg-muted h-3 w-full rounded" />
              <div className="bg-muted h-2 w-full rounded" />
            </CardContent>
          </Card>
        ))}
      </div>
    )
  }

  const stats = [
    {
      label: 'Study hours',
      icon: BookOpen,
      value: data?.study_hours_week ?? 0,
      target: data?.study_target_week ?? 12,
      percent: data?.study_percent ?? 0,
      breakdown: data?.study_breakdown ?? [],
      color: 'text-blue-500',
      bgColor: 'bg-blue-500/10',
      type: 'study' as const,
    },
    {
      label: 'Practice hours',
      icon: Target,
      value: data?.practice_hours_week ?? 0,
      target: data?.practice_target_week ?? 8,
      percent: data?.practice_percent ?? 0,
      breakdown: data?.practice_breakdown ?? [],
      color: 'text-purple-500',
      bgColor: 'bg-purple-500/10',
      type: 'practice' as const,
    },
    {
      label: 'Workouts',
      icon: Dumbbell,
      value: data?.workouts_week ?? 0,
      target: data?.workout_target_week ?? 3,
      percent: data?.workout_percent ?? 0,
      breakdown: [],
      color: 'text-green-500',
      bgColor: 'bg-green-500/10',
      type: 'workout' as const,
    },
    {
      label: 'Active skills',
      icon: Flame,
      value: data?.active_skills_count ?? 0,
      target: data?.skills_target ?? 5,
      percent: Math.min(
        100,
        ((data?.active_skills_count ?? 0) / (data?.skills_target ?? 5)) * 100,
      ),
      breakdown: data?.practice_breakdown ?? [],
      color: 'text-orange-500',
      bgColor: 'bg-orange-500/10',
      type: 'skills' as const,
    },
  ]

  return (
    <div className="grid gap-4 md:grid-cols-4">
      {stats.map((stat) => {
        const Icon = stat.icon
        const hasBreakdown =
          stat.type === 'study' ||
          stat.type === 'practice' ||
          stat.type === 'skills'
        const displayValue =
          stat.type === 'workout' || stat.type === 'skills'
            ? stat.value
            : stat.value.toFixed(1)
        const displayTarget =
          stat.type === 'workout' || stat.type === 'skills'
            ? stat.target
            : stat.target.toFixed(1)

        const CardComponent = (
          <Card
            className={cn(
              'transition-all duration-200',
              hasBreakdown &&
                'cursor-pointer hover:border-primary/50 hover:shadow-md',
            )}
          >
            <CardHeader className="pb-2">
              <div className="flex items-center justify-between">
                <CardTitle className="text-sm font-medium">
                  {stat.label}
                </CardTitle>
                <div className={cn('rounded-md p-1.5', stat.bgColor)}>
                  <Icon className={cn('h-4 w-4', stat.color)} />
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="flex items-baseline gap-1">
                <span className="text-2xl font-bold">{displayValue}</span>
                <span className="text-muted-foreground text-sm">
                  / {displayTarget}
                </span>
              </div>
              <div className="space-y-1">
                <Progress value={stat.percent} className="h-2" />
                <p className="text-muted-foreground text-xs">
                  {Math.round(stat.percent)}% of weekly target
                </p>
              </div>
            </CardContent>
          </Card>
        )

        if (!hasBreakdown) {
          return <div key={stat.label}>{CardComponent}</div>
        }

        return (
          <Popover key={stat.label}>
            <PopoverTrigger asChild>{CardComponent}</PopoverTrigger>
            <PopoverContent className="w-80">
              <PopoverHeader>
                <PopoverTitle className="flex items-center gap-2">
                  <Icon className={cn('h-4 w-4', stat.color)} />
                  {stat.label} Breakdown
                </PopoverTitle>
                <PopoverDescription>
                  Weekly progress by{' '}
                  {stat.type === 'study' ? 'course' : 'skill'}
                </PopoverDescription>
              </PopoverHeader>

              <div className="mt-2 max-h-64 space-y-3 overflow-y-auto">
                {stat.type === 'study' &&
                  data?.study_breakdown?.map((course) => (
                    <div key={course.course_id} className="space-y-1.5">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <div
                            className="h-3 w-3 rounded-full"
                            style={{ backgroundColor: course.color }}
                          />
                          <span className="text-sm font-medium">
                            {course.course_name}
                          </span>
                        </div>
                        <span className="text-muted-foreground text-xs">
                          {course.hours_this_week.toFixed(1)}h /{' '}
                          {course.target_hours.toFixed(1)}h
                        </span>
                      </div>
                      <Progress value={course.percent} className="h-1.5" />
                      {course.current_grade !== undefined &&
                        course.current_grade !== null && (
                          <p className="text-muted-foreground text-xs">
                            Grade: {course.current_grade.toFixed(1)}%
                            {course.target_grade && (
                              <> (target: {course.target_grade}%)</>
                            )}
                          </p>
                        )}
                    </div>
                  ))}

                {(stat.type === 'practice' || stat.type === 'skills') &&
                  data?.practice_breakdown?.map((skill) => (
                    <div key={skill.skill_id} className="space-y-1.5">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <span className="bg-primary/20 text-primary rounded px-1.5 py-0.5 text-xs font-medium">
                            Lv.{skill.current_level}
                          </span>
                          <span className="text-sm font-medium">
                            {skill.skill_name}
                          </span>
                        </div>
                        <span className="text-muted-foreground text-xs">
                          {skill.hours_this_week.toFixed(1)}h /{' '}
                          {skill.target_weekly_hours.toFixed(1)}h
                        </span>
                      </div>
                      <div className="space-y-0.5">
                        <div className="flex items-center gap-2">
                          <span className="text-muted-foreground w-12 text-xs">
                            Week
                          </span>
                          <Progress
                            value={skill.weekly_percent}
                            className="h-1.5 flex-1"
                          />
                        </div>
                        <div className="flex items-center gap-2">
                          <span className="text-muted-foreground w-12 text-xs">
                            Total
                          </span>
                          <Progress
                            value={skill.mastery_percent}
                            className="h-1.5 flex-1"
                          />
                          <span className="text-muted-foreground text-xs">
                            {skill.total_hours.toFixed(0)}h
                          </span>
                        </div>
                      </div>
                    </div>
                  ))}

                {((stat.type === 'study' &&
                  (!data?.study_breakdown ||
                    data.study_breakdown.length === 0)) ||
                  ((stat.type === 'practice' || stat.type === 'skills') &&
                    (!data?.practice_breakdown ||
                      data.practice_breakdown.length === 0))) && (
                  <p className="text-muted-foreground py-4 text-center text-sm">
                    No {stat.type === 'study' ? 'courses' : 'skills'} yet.
                    <br />
                    Add some to track your progress!
                  </p>
                )}
              </div>
            </PopoverContent>
          </Popover>
        )
      })}
    </div>
  )
}
