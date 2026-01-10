import { Link } from '@tanstack/react-router'
import { Trash2Icon } from 'lucide-react'
import { useCourses } from '@/hooks/useCourses'
import { fromNow } from '@/lib/time'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { CourseForm, EditCourseButton } from './course-form'

export function CourseList() {
  const { coursesQuery, deleteCourse } = useCourses()

  const courses = coursesQuery.data ?? []

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Courses</CardTitle>
        <CourseForm />
      </CardHeader>
      <CardContent className="space-y-3">
        {courses.length === 0 && (
          <p className="text-muted-foreground text-sm">No courses yet. Add one to get started!</p>
        )}
        {courses.map((course) => (
          <div
            key={course.id}
            className="border-border bg-muted/60 flex items-center justify-between rounded-md border p-3"
          >
            <Link
              to="/academic/$courseId"
              params={{ courseId: String(course.id) }}
              className="flex items-center gap-3 flex-1 hover:opacity-80"
            >
              <div
                className="h-3 w-3 rounded-full shrink-0"
                style={{ backgroundColor: course.color ?? '#3b82f6' }}
              />
              <div>
                <p className="font-medium">{course.name}</p>
                <p className="text-muted-foreground text-xs">
                  {course.code && `${course.code} · `}Added {fromNow(course.created_at)}
                </p>
              </div>
            </Link>
            <div className="flex items-center gap-1">
              {course.credit_hours && <Badge variant="outline">{course.credit_hours} cr</Badge>}
              <EditCourseButton course={course} />
              <Button
                variant="ghost"
                size="icon-sm"
                onClick={() => deleteCourse.mutate(course.id)}
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

