import { useMemo, useState } from 'react'
import { useParams, Link, useNavigate } from '@tanstack/react-router'
import {
  AlertCircle,
  ArrowLeft,
  BookOpen,
  Calendar as CalendarIcon,
  CheckCircle2Icon,
  Clock,
  FileText,
  GraduationCap,
  MapPin,
  Play,
  Target,
  TrendingUp,
  Trash2Icon,
} from 'lucide-react'
import { useAssignments } from '@/hooks/useAssignments'
import { useCourses, useCourseAnalytics } from '@/hooks/useCourses'
import { useExams } from '@/hooks/useExams'
import {
  formatDate,
  formatDateTime,
  fromNow,
  getDeadlineInfo,
} from '@/lib/time'
import { cn } from '@/lib/utils'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Calendar } from '@/components/ui/calendar'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { AssignmentForm, EditAssignmentButton } from './assignment-form'
import { ExamForm, EditExamButton } from './exam-form'

export function CourseDetail() {
  const { courseId } = useParams({ from: '/academic_/$courseId' })
  const id = Number(courseId)
  const { coursesQuery } = useCourses()
  const { assignmentsQuery, toggleAssignment, deleteAssignment } =
    useAssignments(id)
  const { examsQuery, deleteExam } = useExams(id)
  const { data: analytics, isLoading: analyticsLoading } =
    useCourseAnalytics(id)
  const navigate = useNavigate()

  const [selectedDate, setSelectedDate] = useState<Date | undefined>(undefined)

  const course = useMemo(
    () => coursesQuery.data?.find((c) => c.id === id),
    [coursesQuery.data, id],
  )

  const assignments = assignmentsQuery.data ?? []
  const exams = examsQuery.data ?? []

  // Separate assignments by status
  const pendingAssignments = assignments.filter((a) => !a.is_completed)
  const completedAssignments = assignments.filter((a) => a.is_completed)

  // Get dates with events for calendar
  const eventDates = useMemo(() => {
    const dates: Date[] = []
    for (const a of assignments) {
      if (a.due_date) dates.push(new Date(a.due_date))
    }
    for (const e of exams) {
      if (e.exam_date) dates.push(new Date(e.exam_date))
    }
    return dates
  }, [assignments, exams])

  const handleStartSession = () => {
    navigate({
      to: '/dashboard',
      search: { courseId: id },
    })
  }

  // Get items for selected date
  const selectedDateItems = useMemo(() => {
    if (!selectedDate) return { assignments: [], exams: [] }
    const dateStr = formatDate(selectedDate, 'yyyy-MM-dd')
    return {
      assignments: assignments.filter(
        (a) => a.due_date && formatDate(a.due_date, 'yyyy-MM-dd') === dateStr,
      ),
      exams: exams.filter(
        (e) => e.exam_date && formatDate(e.exam_date, 'yyyy-MM-dd') === dateStr,
      ),
    }
  }, [selectedDate, assignments, exams])

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center gap-4">
        <Link to="/academic">
          <Button variant="ghost" size="icon-sm">
            <ArrowLeft className="h-4 w-4" />
          </Button>
        </Link>
        <div className="flex flex-1 items-center gap-3">
          <div
            className="h-5 w-5 rounded-full"
            style={{ backgroundColor: course?.color ?? '#3b82f6' }}
          />
          <div>
            <h1 className="text-xl font-semibold">
              {course?.name ?? 'Course'}
            </h1>
            <p className="text-muted-foreground text-sm">
              {course?.code ? `${course.code} · ` : ''}
              {course?.credit_hours
                ? `${course.credit_hours} credits`
                : 'No credits set'}
            </p>
          </div>
        </div>
        <Button onClick={handleStartSession}>
          <Play className="mr-1 h-4 w-4" />
          Study
        </Button>
      </div>

      {/* Quick Stats Row */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardContent className="pt-4">
            <div className="flex items-center gap-2">
              <div className="bg-blue-500/10 rounded-md p-2">
                <Clock className="h-4 w-4 text-blue-500" />
              </div>
              <div>
                <p className="text-muted-foreground text-xs">This Week</p>
                <p className="text-lg font-semibold">
                  {analytics?.hours_this_week.toFixed(1) ?? '0'}h
                  <span className="text-muted-foreground text-sm font-normal">
                    {' '}
                    / {course?.target_weekly_hours ?? 6}h
                  </span>
                </p>
              </div>
            </div>
            <Progress
              value={analytics?.weekly_percent ?? 0}
              className="mt-2 h-1.5"
            />
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-4">
            <div className="flex items-center gap-2">
              <div className="bg-purple-500/10 rounded-md p-2">
                <BookOpen className="h-4 w-4 text-purple-500" />
              </div>
              <div>
                <p className="text-muted-foreground text-xs">Total Study</p>
                <p className="text-lg font-semibold">
                  {analytics?.total_hours.toFixed(1) ?? '0'}h
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-4">
            <div className="flex items-center gap-2">
              <div className="bg-green-500/10 rounded-md p-2">
                <Target className="h-4 w-4 text-green-500" />
              </div>
              <div>
                <p className="text-muted-foreground text-xs">Current Grade</p>
                <p className="text-lg font-semibold">
                  {course?.current_grade !== undefined &&
                  course?.current_grade !== null
                    ? `${course.current_grade.toFixed(1)}%`
                    : 'N/A'}
                  {course?.target_grade && (
                    <span className="text-muted-foreground text-sm font-normal">
                      {' '}
                      / {course.target_grade}%
                    </span>
                  )}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-4">
            <div className="flex items-center gap-2">
              <div className="bg-amber-500/10 rounded-md p-2">
                <FileText className="h-4 w-4 text-amber-500" />
              </div>
              <div>
                <p className="text-muted-foreground text-xs">Assignments</p>
                <p className="text-lg font-semibold">
                  {completedAssignments.length}
                  <span className="text-muted-foreground text-sm font-normal">
                    {' '}
                    / {assignments.length}
                  </span>
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Tabs */}
      <Tabs defaultValue="overview" className="space-y-4">
        <TabsList variant="line">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="assignments">
            Assignments
            {pendingAssignments.length > 0 && (
              <Badge variant="secondary" className="ml-1.5 text-[10px]">
                {pendingAssignments.length}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="exams">
            Exams
            {exams.length > 0 && (
              <Badge variant="secondary" className="ml-1.5 text-[10px]">
                {exams.length}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="calendar">Calendar</TabsTrigger>
          <TabsTrigger value="analytics">Analytics</TabsTrigger>
        </TabsList>

        {/* Overview Tab */}
        <TabsContent value="overview" className="space-y-4">
          {/* Upcoming Deadlines */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <AlertCircle className="h-4 w-4" />
                Upcoming Deadlines
              </CardTitle>
            </CardHeader>
            <CardContent>
              {pendingAssignments.length === 0 && exams.length === 0 ? (
                <p className="text-muted-foreground text-sm">
                  No upcoming deadlines. You're all caught up!
                </p>
              ) : (
                <div className="space-y-3">
                  {/* Show next 5 upcoming items */}
                  {[
                    ...pendingAssignments.map((a) => ({
                      type: 'assignment' as const,
                      title: a.title,
                      date: a.due_date,
                      id: a.id,
                    })),
                    ...exams.map((e) => ({
                      type: 'exam' as const,
                      title: e.title,
                      date: e.exam_date,
                      id: e.id,
                    })),
                  ]
                    .filter((item) => item.date)
                    .sort(
                      (a, b) =>
                        new Date(a.date!).getTime() -
                        new Date(b.date!).getTime(),
                    )
                    .slice(0, 5)
                    .map((item) => {
                      const deadline = getDeadlineInfo(item.date)
                      return (
                        <div
                          key={`${item.type}-${item.id}`}
                          className="flex items-center justify-between"
                        >
                          <div className="flex items-center gap-2">
                            {item.type === 'exam' ? (
                              <GraduationCap className="text-muted-foreground h-4 w-4" />
                            ) : (
                              <FileText className="text-muted-foreground h-4 w-4" />
                            )}
                            <span className="text-sm font-medium">
                              {item.title}
                            </span>
                            <Badge variant="outline" className="text-[10px]">
                              {item.type}
                            </Badge>
                          </div>
                          <span
                            className={cn('text-sm', deadline?.urgencyColor)}
                          >
                            {deadline?.label}
                          </span>
                        </div>
                      )
                    })}
                </div>
              )}
            </CardContent>
          </Card>

          {/* Course Info */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Course Details</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Weekly Target</span>
                <span>{course?.target_weekly_hours ?? 6} hours</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Total Sessions</span>
                <span>{analytics?.sessions_count ?? 0}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Avg Session</span>
                <span>
                  {analytics?.avg_session_duration
                    ? `${analytics.avg_session_duration.toFixed(0)} min`
                    : 'N/A'}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Created</span>
                <span>{fromNow(course?.created_at)}</span>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Assignments Tab */}
        <TabsContent value="assignments" className="space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-medium">Assignments</h2>
            <AssignmentForm courseId={id} />
          </div>

          {assignments.length === 0 ? (
            <Card>
              <CardContent className="py-8 text-center">
                <FileText className="text-muted-foreground mx-auto mb-3 h-12 w-12" />
                <p className="text-muted-foreground text-sm">
                  No assignments yet. Add one to get started!
                </p>
              </CardContent>
            </Card>
          ) : (
            <div className="space-y-4">
              {/* Pending */}
              {pendingAssignments.length > 0 && (
                <div className="space-y-2">
                  <h3 className="text-muted-foreground text-sm font-medium">
                    Pending ({pendingAssignments.length})
                  </h3>
                  {pendingAssignments.map((assignment) => (
                    <AssignmentCard
                      key={assignment.id}
                      assignment={assignment}
                      courseId={id}
                      onToggle={() => toggleAssignment.mutate(assignment.id)}
                      onDelete={() => deleteAssignment.mutate(assignment.id)}
                    />
                  ))}
                </div>
              )}

              {/* Completed */}
              {completedAssignments.length > 0 && (
                <div className="space-y-2">
                  <h3 className="text-muted-foreground text-sm font-medium">
                    Completed ({completedAssignments.length})
                  </h3>
                  {completedAssignments.map((assignment) => (
                    <AssignmentCard
                      key={assignment.id}
                      assignment={assignment}
                      courseId={id}
                      onToggle={() => toggleAssignment.mutate(assignment.id)}
                      onDelete={() => deleteAssignment.mutate(assignment.id)}
                    />
                  ))}
                </div>
              )}
            </div>
          )}
        </TabsContent>

        {/* Exams Tab */}
        <TabsContent value="exams" className="space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-medium">Exams</h2>
            <ExamForm courseId={id} />
          </div>

          {exams.length === 0 ? (
            <Card>
              <CardContent className="py-8 text-center">
                <GraduationCap className="text-muted-foreground mx-auto mb-3 h-12 w-12" />
                <p className="text-muted-foreground text-sm">
                  No exams yet. Add one to track your exam schedule!
                </p>
              </CardContent>
            </Card>
          ) : (
            <div className="space-y-2">
              {exams
                .sort((a, b) => {
                  if (!a.exam_date) return 1
                  if (!b.exam_date) return -1
                  return (
                    new Date(a.exam_date).getTime() -
                    new Date(b.exam_date).getTime()
                  )
                })
                .map((exam) => (
                  <ExamCard
                    key={exam.id}
                    exam={exam}
                    courseId={id}
                    onDelete={() => deleteExam.mutate(exam.id)}
                  />
                ))}
            </div>
          )}
        </TabsContent>

        {/* Calendar Tab */}
        <TabsContent value="calendar" className="space-y-4">
          <div className="grid gap-4 md:grid-cols-[300px_1fr]">
            <Card>
              <CardContent className="p-2">
                <Calendar
                  mode="single"
                  selected={selectedDate}
                  onSelect={setSelectedDate}
                  modifiers={{
                    hasEvent: eventDates,
                  }}
                  modifiersStyles={{
                    hasEvent: {
                      fontWeight: 'bold',
                      textDecoration: 'underline',
                      textDecorationColor: course?.color ?? '#3b82f6',
                    },
                  }}
                />
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="text-base">
                  {selectedDate
                    ? formatDate(selectedDate, 'EEEE, MMMM d, yyyy')
                    : 'Select a date'}
                </CardTitle>
              </CardHeader>
              <CardContent>
                {!selectedDate ? (
                  <p className="text-muted-foreground text-sm">
                    Click on a date to see assignments and exams.
                  </p>
                ) : selectedDateItems.assignments.length === 0 &&
                  selectedDateItems.exams.length === 0 ? (
                  <p className="text-muted-foreground text-sm">
                    No events on this date.
                  </p>
                ) : (
                  <div className="space-y-3">
                    {selectedDateItems.exams.map((exam) => (
                      <div
                        key={exam.id}
                        className="bg-red-500/10 flex items-center gap-2 rounded-md p-2"
                      >
                        <GraduationCap className="h-4 w-4 text-red-500" />
                        <div>
                          <p className="text-sm font-medium">{exam.title}</p>
                          {exam.exam_date && (
                            <p className="text-muted-foreground text-xs">
                              {formatDateTime(exam.exam_date)}
                            </p>
                          )}
                        </div>
                      </div>
                    ))}
                    {selectedDateItems.assignments.map((assignment) => (
                      <div
                        key={assignment.id}
                        className="bg-amber-500/10 flex items-center gap-2 rounded-md p-2"
                      >
                        <FileText className="h-4 w-4 text-amber-500" />
                        <div>
                          <p className="text-sm font-medium">
                            {assignment.title}
                          </p>
                          <Badge
                            variant={
                              assignment.priority === 'high'
                                ? 'destructive'
                                : 'secondary'
                            }
                            className="text-[10px]"
                          >
                            {assignment.priority ?? 'medium'}
                          </Badge>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        {/* Analytics Tab */}
        <TabsContent value="analytics" className="space-y-4">
          {analyticsLoading ? (
            <Card>
              <CardContent className="py-8 text-center">
                <p className="text-muted-foreground text-sm">
                  Loading analytics...
                </p>
              </CardContent>
            </Card>
          ) : (
            <>
              {/* Weekly History */}
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 text-base">
                    <TrendingUp className="h-4 w-4" />
                    Weekly Study History
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  {analytics?.weekly_history &&
                  analytics.weekly_history.length > 0 ? (
                    <div className="space-y-2">
                      {analytics.weekly_history.slice(-8).map((week) => (
                        <div
                          key={week.week_start}
                          className="flex items-center gap-3"
                        >
                          <span className="text-muted-foreground w-24 text-xs">
                            {formatDate(week.week_start, 'MMM d')}
                          </span>
                          <div className="flex-1">
                            <Progress
                              value={Math.min(
                                100,
                                (week.hours /
                                  (course?.target_weekly_hours ?? 6)) *
                                  100,
                              )}
                              className="h-2"
                            />
                          </div>
                          <span className="w-12 text-right text-xs">
                            {week.hours.toFixed(1)}h
                          </span>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <p className="text-muted-foreground text-sm">
                      No study history yet. Start a session to track your
                      progress!
                    </p>
                  )}
                </CardContent>
              </Card>

              {/* Stats Summary */}
              <Card>
                <CardHeader>
                  <CardTitle className="text-base">Statistics</CardTitle>
                </CardHeader>
                <CardContent className="grid gap-4 md:grid-cols-3">
                  <div className="text-center">
                    <p className="text-3xl font-bold">
                      {analytics?.sessions_count ?? 0}
                    </p>
                    <p className="text-muted-foreground text-sm">
                      Total Sessions
                    </p>
                  </div>
                  <div className="text-center">
                    <p className="text-3xl font-bold">
                      {analytics?.total_hours.toFixed(1) ?? '0'}h
                    </p>
                    <p className="text-muted-foreground text-sm">Total Hours</p>
                  </div>
                  <div className="text-center">
                    <p className="text-3xl font-bold">
                      {analytics?.avg_session_duration?.toFixed(0) ?? '0'}
                    </p>
                    <p className="text-muted-foreground text-sm">
                      Avg Minutes/Session
                    </p>
                  </div>
                </CardContent>
              </Card>
            </>
          )}
        </TabsContent>
      </Tabs>
    </div>
  )
}

// Assignment Card Component
function AssignmentCard({
  assignment,
  courseId,
  onToggle,
  onDelete,
}: {
  assignment: {
    id: number
    title: string
    description?: string
    due_date?: string
    priority?: string
    is_completed?: number
  }
  courseId: number
  onToggle: () => void
  onDelete: () => void
}) {
  const deadline = getDeadlineInfo(assignment.due_date)

  return (
    <Card>
      <CardContent className="flex items-start justify-between p-3">
        <div className="flex flex-1 items-start gap-3">
          <button
            type="button"
            className="text-muted-foreground hover:text-primary mt-0.5 transition-colors"
            onClick={onToggle}
          >
            <CheckCircle2Icon
              className={cn(
                'h-5 w-5',
                assignment.is_completed && 'text-primary',
              )}
            />
          </button>
          <div className="flex-1 space-y-1">
            <p
              className={cn(
                'font-medium',
                assignment.is_completed && 'line-through opacity-60',
              )}
            >
              {assignment.title}
            </p>
            {assignment.description && (
              <p className="text-muted-foreground text-xs">
                {assignment.description}
              </p>
            )}
            {assignment.due_date && (
              <div className="flex items-center gap-2">
                <CalendarIcon className="text-muted-foreground h-3 w-3" />
                <span className={cn('text-xs', deadline?.urgencyColor)}>
                  {deadline?.label}
                </span>
              </div>
            )}
          </div>
        </div>
        <div className="flex items-center gap-1">
          <Badge
            variant={
              assignment.priority === 'high' ? 'destructive' : 'secondary'
            }
            className="text-[10px]"
          >
            {assignment.priority ?? 'medium'}
          </Badge>
          <EditAssignmentButton
            courseId={courseId}
            assignment={assignment as any}
          />
          <Button variant="ghost" size="icon-sm" onClick={onDelete}>
            <Trash2Icon className="text-destructive h-3 w-3" />
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}

// Exam Card Component
function ExamCard({
  exam,
  courseId,
  onDelete,
}: {
  exam: {
    id: number
    title: string
    exam_date?: string
    location?: string
    duration_minutes?: number
    weight?: number
    grade?: number
    notes?: string
  }
  courseId: number
  onDelete: () => void
}) {
  const deadline = getDeadlineInfo(exam.exam_date)

  return (
    <Card>
      <CardContent className="p-4">
        <div className="flex items-start justify-between">
          <div className="flex items-start gap-3">
            <div className="bg-red-500/10 mt-0.5 rounded-md p-2">
              <GraduationCap className="h-4 w-4 text-red-500" />
            </div>
            <div className="space-y-1">
              <p className="font-medium">{exam.title}</p>
              {exam.exam_date && (
                <div className="flex items-center gap-2">
                  <CalendarIcon className="text-muted-foreground h-3 w-3" />
                  <span className={cn('text-sm', deadline?.urgencyColor)}>
                    {formatDateTime(exam.exam_date)}
                  </span>
                  {deadline && !deadline.isPast && (
                    <Badge
                      variant={
                        deadline.urgency === 'critical'
                          ? 'destructive'
                          : 'outline'
                      }
                      className="text-[10px]"
                    >
                      {deadline.label}
                    </Badge>
                  )}
                </div>
              )}
              <div className="text-muted-foreground flex flex-wrap items-center gap-3 text-xs">
                {exam.location && (
                  <span className="flex items-center gap-1">
                    <MapPin className="h-3 w-3" />
                    {exam.location}
                  </span>
                )}
                {exam.duration_minutes && (
                  <span className="flex items-center gap-1">
                    <Clock className="h-3 w-3" />
                    {exam.duration_minutes} min
                  </span>
                )}
                {exam.weight && <span>Weight: {exam.weight}%</span>}
                {exam.grade !== undefined && exam.grade !== null && (
                  <Badge variant="outline">Grade: {exam.grade}%</Badge>
                )}
              </div>
              {exam.notes && (
                <p className="text-muted-foreground text-xs">{exam.notes}</p>
              )}
            </div>
          </div>
          <div className="flex items-center gap-1">
            <EditExamButton courseId={courseId} exam={exam as any} />
            <Button variant="ghost" size="icon-sm" onClick={onDelete}>
              <Trash2Icon className="text-destructive h-3 w-3" />
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
