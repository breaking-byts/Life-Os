import { createFileRoute } from '@tanstack/react-router'
import { CourseList } from '@/components/academic/course-list'
import { MainLayout } from '@/components/layout/main-layout'

export const Route = createFileRoute('/academic')({
  component: AcademicPage,
})

function AcademicPage() {
  return (
    <MainLayout>
      <div className="space-y-6">
        <div className="space-y-2">
          <h1 className="text-2xl font-semibold">Academic</h1>
          <p className="text-muted-foreground text-sm">
            Track courses and assignments. Create and manage course data via the
            Tauri backend hooks.
          </p>
        </div>
        <CourseList />
      </div>
    </MainLayout>
  )
}
