import { TrophyIcon } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { usePersonalRecords } from '@/hooks/useStats'
import { formatPRValue } from '@/lib/workout-utils'
import { fromNow } from '@/lib/time'

export function PersonalRecords() {
  const { data: prs, isLoading } = usePersonalRecords()

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <TrophyIcon className="h-4 w-4 text-yellow-500" />
            Personal Records
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {[1, 2, 3].map((i) => (
              <div key={i} className="h-14 animate-pulse rounded-md bg-muted" />
            ))}
          </div>
        </CardContent>
      </Card>
    )
  }

  const recentPRs = prs?.slice(0, 5) ?? []

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-base">
          <TrophyIcon className="h-4 w-4 text-yellow-500" />
          Personal Records
        </CardTitle>
      </CardHeader>
      <CardContent>
        {recentPRs.length === 0 ? (
          <p className="text-muted-foreground text-sm">
            No personal records yet. Start logging workouts to track your PRs!
          </p>
        ) : (
          <div className="space-y-3">
            {recentPRs.map((pr) => (
              <div
                key={pr.id}
                className="flex items-center justify-between rounded-md border border-border bg-muted/40 p-3"
              >
                <div className="flex items-center gap-3">
                  <div className="flex h-8 w-8 items-center justify-center rounded-full bg-yellow-500/10">
                    <TrophyIcon className="h-4 w-4 text-yellow-500" />
                  </div>
                  <div>
                    <p className="text-sm font-medium">{pr.exercise_name}</p>
                    <p className="text-xs text-muted-foreground">
                      {fromNow(pr.achieved_at)}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <Badge variant="secondary" className="text-xs capitalize">
                    {pr.pr_type}
                  </Badge>
                  <span className="font-semibold text-sm">
                    {formatPRValue(pr.pr_type, pr.value)}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
