import { Link, useNavigate } from '@tanstack/react-router'
import {
  AlertCircle,
  BookOpen,
  Calendar,
  Clock,
  Play,
  Trash2Icon,
} from 'lucide-react'
import { useCourses, useCoursesWithProgress } from '@/hooks/useCourses'
import { fromNow } from '@/lib/time'
import { cn } from '@/lib/utils'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { CourseForm, EditCourseButton } from './course-form'

export function CourseList() {
  const { deleteCourse } = useCourses()
  const { data: courses, isLoading, error } = useCoursesWithProgress()
  const navigate = useNavigate()

  const handleStartSession = (courseId: number) => {
    navigate({
      to: '/dashboard',
      search: { courseId },
    })
  }

  if (isLoading) {
    return (
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle>Courses</CardTitle>
          <CourseForm />
        </CardHeader>
        <CardContent className="space-y-3">
          {[1, 2, 3].map((i) => (
            <div
              key={i}
              className="border-border bg-muted/60 animate-pulse rounded-md border p-4"
            >
              <div className="flex items-center gap-3">
                <div className="bg-muted-foreground/20 h-4 w-4 rounded-full" />
                <div className="flex-1 space-y-2">
                  <div className="bg-muted-foreground/20 h-4 w-32 rounded" />
                  <div className="bg-muted-foreground/20 h-2 w-full rounded" />
                </div>
              </div>
            </div>
          ))}
        </CardContent>
      </Card>
    )
  }

  if (error) {
    return (
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle>Courses</CardTitle>
          <CourseForm />
        </CardHeader>
        <CardContent>
          <p className="text-destructive text-sm">
            Failed to load courses. Please try again.
          </p>
        </CardContent>
      </Card>
    )
  }

  const courseList = courses ?? []

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle className="flex items-center gap-2">
          <BookOpen className="h-5 w-5" />
          Courses
        </CardTitle>
        <CourseForm />
      </CardHeader>
      <CardContent className="space-y-3">
        {courseList.length === 0 && (
          <div className="py-8 text-center">
            <BookOpen className="text-muted-foreground mx-auto mb-3 h-12 w-12" />
            <p className="text-muted-foreground text-sm">
              No courses yet. Add one to get started!
            </p>
          </div>
        )}
        {courseList.map((course) => {
          const progressPercent = Math.min(100, course.weekly_percent ?? 0)
          const hasOverdue = (course.overdue_assignments ?? 0) > 0
          const hasUpcoming = (course.upcoming_assignments ?? 0) > 0
          const targetHours = course.target_weekly_hours ?? 0

          return (
            <div
              key={course.id}
              className="border-border bg-muted/40 hover:bg-muted/60 group rounded-lg border p-4 transition-all"
            >
              {/* Top row: Course info + actions */}
              <div className="flex items-start justify-between gap-4">
                <Link
                  to="/academic/$courseId"
                  params={{ courseId: String(course.id) }}
                  className="flex flex-1 items-start gap-3 hover:opacity-80"
                >
                  <div
                    className="mt-1 h-4 w-4 shrink-0 rounded-full"
                    style={{ backgroundColor: course.color ?? '#3b82f6' }}
                  />
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <p className="truncate font-medium">{course.name}</p>
                      {course.code && (
                        <Badge variant="outline" className="text-xs">
                          {course.code}
                        </Badge>
                      )}
                    </div>
                    <div className="text-muted-foreground mt-0.5 flex flex-wrap items-center gap-2 text-xs">
                      {course.credit_hours && (
                        <span>{course.credit_hours} credits</span>
                      )}
                      {course.current_grade !== undefined &&
                        course.current_grade !== null && (
                          <span
                            className={cn(
                              course.target_grade &&
                                course.current_grade < course.target_grade
                                ? 'text-amber-500'
                                : 'text-green-500',
                            )}
                          >
                            {course.current_grade.toFixed(1)}%
                            {course.target_grade && (
                              <span className="text-muted-foreground">
                                {' '}
                                / {course.target_grade}%
                              </span>
                            )}
                          </span>
                        )}
                      <span className="text-muted-foreground/60">
                        Added {fromNow(course.created_at)}
                      </span>
                    </div>
                  </div>
                </Link>

                {/* Actions */}
                <div className="flex shrink-0 items-center gap-1">
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant="ghost"
                          size="icon-sm"
                          className="text-primary hover:text-primary hover:bg-primary/10"
                          onClick={(e) => {
                            e.preventDefault()
                            handleStartSession(course.id)
                          }}
                        >
                          <Play className="h-4 w-4" />
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>Start Study Session</TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                  <EditCourseButton course={course} />
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    onClick={() => deleteCourse.mutate(course.id)}
                    disabled={deleteCourse.isPending}
                  >
                    <Trash2Icon className="text-destructive h-3.5 w-3.5" />
                  </Button>
                </div>
              </div>

              {/* Progress bar section */}
              {targetHours > 0 && (
                <div className="mt-3 space-y-1.5">
                  <div className="flex items-center justify-between text-xs">
                    <div className="flex items-center gap-1.5">
                      <Clock className="text-muted-foreground h-3 w-3" />
                      <span className="text-muted-foreground">This week</span>
                    </div>
                    <span className="font-medium">
                      {course.hours_this_week.toFixed(1)}h /{' '}
                      {targetHours.toFixed(1)}h
                      <span className="text-muted-foreground ml-1">
                        ({Math.round(progressPercent)}%)
                      </span>
                    </span>
                  </div>
                  <Progress
                    value={progressPercent}
                    className={cn(
                      'h-2',
                      progressPercent >= 100 && '[&>div]:bg-green-500',
                      progressPercent >= 75 &&
                        progressPercent < 100 &&
                        '[&>div]:bg-blue-500',
                      progressPercent < 75 && '[&>div]:bg-amber-500',
                    )}
                  />
                </div>
              )}

              {/* Bottom row: Assignment badges */}
              {(hasOverdue || hasUpcoming) && (
                <div className="mt-3 flex flex-wrap items-center gap-2">
                  {hasOverdue && (
                    <Badge
                      variant="destructive"
                      className="flex items-center gap-1 text-xs"
                    >
                      <AlertCircle className="h-3 w-3" />
                      {course.overdue_assignments} overdue
                    </Badge>
                  )}
                  {hasUpcoming && (
                    <Badge
                      variant="secondary"
                      className="flex items-center gap-1 text-xs"
                    >
                      <Calendar className="h-3 w-3" />
                      {course.upcoming_assignments} upcoming
                    </Badge>
                  )}
                </div>
              )}

              {/* Total hours indicator */}
              {course.total_hours > 0 && (
                <div className="text-muted-foreground mt-2 text-xs">
                  Total: {course.total_hours.toFixed(1)} hours logged
                </div>
              )}
            </div>
          )
        })}
      </CardContent>
    </Card>
  )
}
