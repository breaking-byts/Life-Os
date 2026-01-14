import { createFileRoute } from '@tanstack/react-router'
import { z } from 'zod'
import { AgentBigThree } from '@/components/dashboard/agent-big-three'
import { AgentInsight } from '@/components/dashboard/agent-insight'
import { AgentRecommendations } from '@/components/dashboard/agent-recommendations'
import { QuickStats } from '@/components/dashboard/quick-stats'
import { StreakDisplay } from '@/components/dashboard/streak-display'
import { TodayView } from '@/components/dashboard/today-view'
import { MainLayout } from '@/components/layout/main-layout'
import { PomodoroTimer } from '@/components/ui/pomodoro-timer'

const dashboardSearchSchema = z.object({
  courseId: z.number().optional(),
})

export const Route = createFileRoute('/dashboard')({
  component: DashboardPage,
  validateSearch: dashboardSearchSchema,
})

function DashboardPage() {
  const { courseId } = Route.useSearch()

  return (
    <MainLayout>
      <div className="space-y-6">
        <QuickStats />
        <div className="grid gap-4 md:grid-cols-3">
          <div className="md:col-span-2 space-y-4">
            <AgentRecommendations />
            <TodayView />
            <AgentBigThree />
          </div>
          <div className="space-y-4">
            <PomodoroTimer initialCourseId={courseId} />
            <StreakDisplay />
            <AgentInsight />
          </div>
        </div>
      </div>
    </MainLayout>
  )
}
