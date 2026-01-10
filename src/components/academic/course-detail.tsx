import { useMemo } from 'react'
import { useParams } from '@tanstack/react-router'
import { CheckCircle2Icon, Trash2Icon } from 'lucide-react'
import { useAssignments } from '@/hooks/useAssignments'
import { useCourses } from '@/hooks/useCourses'
import { fromNow } from '@/lib/time'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { AssignmentForm, EditAssignmentButton } from './assignment-form'

export function CourseDetail() {
  const { courseId } = useParams({ from: '/academic/$courseId' })
  const id = Number(courseId)
  const { coursesQuery } = useCourses()
  const { assignmentsQuery, toggleAssignment, deleteAssignment } = useAssignments(id)

  const course = useMemo(
    () => coursesQuery.data?.find((c) => c.id === id),
    [coursesQuery.data, id],
  )

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div
                className="h-4 w-4 rounded-full"
                style={{ backgroundColor: course?.color ?? '#3b82f6' }}
              />
              <div>
                <CardTitle>{course?.name ?? 'Course'}</CardTitle>
                <p className="text-muted-foreground text-sm">
                  {course?.code ? `Code ${course.code}` : 'No code set'}
                </p>
              </div>
            </div>
            {course?.credit_hours && (
              <Badge variant="outline">{course.credit_hours} credits</Badge>
            )}
          </div>
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          Target: {course?.target_weekly_hours ?? 6} hours/week · Created {fromNow(course?.created_at)}
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle>Assignments</CardTitle>
          <AssignmentForm courseId={id} />
        </CardHeader>
        <CardContent className="space-y-3">
          {assignmentsQuery.data?.length === 0 && (
            <p className="text-muted-foreground text-sm">No assignments yet. Add one to get started!</p>
          )}
          {assignmentsQuery.data?.map((assignment) => (
            <div
              key={assignment.id}
              className="border-border flex items-start justify-between rounded-md border p-3"
            >
              <div className="space-y-1 flex-1">
                <div className="flex items-center gap-2">
                  <button
                    type="button"
                    className="text-muted-foreground hover:text-primary transition-colors"
                    onClick={() => toggleAssignment.mutate(assignment.id)}
                  >
                    <CheckCircle2Icon
                      className={
                        assignment.is_completed ? 'text-primary h-4 w-4' : 'h-4 w-4'
                      }
                    />
                  </button>
                  <div>
                    <p className={`font-medium ${assignment.is_completed ? 'line-through opacity-60' : ''}`}>
                      {assignment.title}
                    </p>
                    {assignment.due_date && (
                      <p className="text-muted-foreground text-xs">
                        Due {fromNow(assignment.due_date)}
                      </p>
                    )}
                  </div>
                </div>
                {assignment.description && (
                  <p className="text-muted-foreground text-xs pl-6">{assignment.description}</p>
                )}
              </div>
              <div className="flex items-center gap-1">
                <Badge
                  variant={assignment.priority === 'high' ? 'destructive' : 'secondary'}
                  className="text-[10px]"
                >
                  {assignment.priority ?? 'medium'}
                </Badge>
                <EditAssignmentButton courseId={id} assignment={assignment} />
                <Button
                  variant="ghost"
                  size="icon-sm"
                  onClick={() => deleteAssignment.mutate(assignment.id)}
                >
                  <Trash2Icon className="h-3 w-3 text-destructive" />
                </Button>
              </div>
            </div>
          ))}
        </CardContent>
      </Card>
    </div>
  )
}

