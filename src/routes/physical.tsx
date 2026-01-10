import { createFileRoute } from '@tanstack/react-router'
import { WorkoutList } from '@/components/physical/workout-list'
import { MainLayout } from '@/components/layout/main-layout'

export const Route = createFileRoute('/physical')({
  component: PhysicalPage,
})

function PhysicalPage() {
  return (
    <MainLayout>
      <div className="space-y-6">
        <div className="space-y-2">
          <h1 className="text-2xl font-semibold">Physical</h1>
          <p className="text-muted-foreground text-sm">
            Workouts and exercises powered by Tauri commands.
          </p>
        </div>
        <WorkoutList />
      </div>
    </MainLayout>
  )
}
