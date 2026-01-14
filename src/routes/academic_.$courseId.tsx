import { createFileRoute } from '@tanstack/react-router'
import { CourseDetail } from '@/components/academic/course-detail'
import { MainLayout } from '@/components/layout/main-layout'

export const Route = createFileRoute('/academic_/$courseId')({
  component: CourseDetailPage,
})

function CourseDetailPage() {
  return (
    <MainLayout>
      <div className="space-y-6">
        <CourseDetail />
      </div>
    </MainLayout>
  )
}
