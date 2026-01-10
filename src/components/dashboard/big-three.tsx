import { useQuery } from '@tanstack/react-query'
import { CalendarIcon, CheckCircleIcon } from 'lucide-react'
import { tauri } from '@/lib/tauri'
import { fromNow } from '@/lib/time'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'

export function BigThree() {
  const { data: assignments = [], isLoading } = useQuery({
    queryKey: ['assignments'],
    queryFn: () => tauri.getAssignments(),
  })

  // Filter incomplete assignments and sort by due date
  const upcomingAssignments = assignments
    .filter((a) => !a.is_completed && a.due_date)
    .sort((a, b) => {
      const dateA = a.due_date ? new Date(a.due_date).getTime() : Infinity
      const dateB = b.due_date ? new Date(b.due_date).getTime() : Infinity
      return dateA - dateB
    })
    .slice(0, 3)

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Today&apos;s Big 3</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground text-sm">Loading...</p>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Top 3 Due Soon</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {upcomingAssignments.length === 0 ? (
          <div className="text-center py-4">
            <CheckCircleIcon className="mx-auto h-8 w-8 text-green-500 mb-2" />
            <p className="text-muted-foreground text-sm">All caught up! No upcoming assignments.</p>
          </div>
        ) : (
          upcomingAssignments.map((assignment, index) => {
            const dueDate = assignment.due_date ? new Date(assignment.due_date) : null
            const isOverdue = dueDate && dueDate < new Date()

            return (
              <div
                key={assignment.id}
                className={`border-border flex items-center justify-between rounded-md border p-3 ${isOverdue ? 'border-red-500/50 bg-red-500/5' : 'bg-muted/60'
                  }`}
              >
                <div className="flex items-center gap-3">
                  <span className="text-muted-foreground font-mono text-sm">{index + 1}.</span>
                  <div>
                    <p className="font-medium">{assignment.title}</p>
                    <div className="flex items-center gap-2 text-xs text-muted-foreground">
                      <CalendarIcon className="h-3 w-3" />
                      <span className={isOverdue ? 'text-red-500' : ''}>
                        {dueDate ? fromNow(dueDate) : 'No due date'}
                      </span>
                    </div>
                  </div>
                </div>
                <Badge variant={isOverdue ? 'destructive' : 'outline'}>
                  {assignment.priority || 'medium'}
                </Badge>
              </div>
            )
          })
        )}
      </CardContent>
    </Card>
  )
}
