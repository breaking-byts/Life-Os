import { createFileRoute } from '@tanstack/react-router'
import { AgentInsight } from '@/components/dashboard/agent-insight'
import { BigThree } from '@/components/dashboard/big-three'
import { QuickStats } from '@/components/dashboard/quick-stats'
import { StreakDisplay } from '@/components/dashboard/streak-display'
import { TodayView } from '@/components/dashboard/today-view'
import { MainLayout } from '@/components/layout/main-layout'
import { PomodoroTimer } from '@/components/ui/pomodoro-timer'

export const Route = createFileRoute('/dashboard')({
  component: DashboardPage,
})

function DashboardPage() {
  return (
    <MainLayout>
      <div className="space-y-6">
        <QuickStats />
        <div className="grid gap-4 md:grid-cols-3">
          <div className="md:col-span-2 space-y-4">
            <TodayView />
            <BigThree />
          </div>
          <div className="space-y-4">
            <PomodoroTimer />
            <StreakDisplay />
            <AgentInsight />
          </div>
        </div>
      </div>
    </MainLayout>
  )
}

