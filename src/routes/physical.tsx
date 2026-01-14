import { createFileRoute } from '@tanstack/react-router'
import { MainLayout } from '@/components/layout/main-layout'
import { GymSettings } from '@/components/physical/gym-settings'
import { QuickStats } from '@/components/physical/quick-stats'
import { WorkoutHeatmap } from '@/components/physical/workout-heatmap'
import { WorkoutList } from '@/components/physical/workout-list'
import { PersonalRecords } from '@/components/physical/personal-records'

export const Route = createFileRoute('/physical')({
  component: PhysicalPage,
})

function PhysicalPage() {
  return (
    <MainLayout>
      <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="space-y-2">
            <h1 className="text-2xl font-semibold">Physical</h1>
            <p className="text-muted-foreground text-sm">
              Track workouts, exercises, and fitness progress
            </p>
          </div>
          <GymSettings />
        </div>

        {/* Quick Stats Row */}
        <QuickStats />

        {/* Heatmap */}
        <WorkoutHeatmap />

        {/* Two Column Layout */}
        <div className="grid gap-6 lg:grid-cols-3">
          <div className="lg:col-span-2">
            <WorkoutList />
          </div>
          <div>
            <PersonalRecords />
          </div>
        </div>
      </div>
    </MainLayout>
  )
}
