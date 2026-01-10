import { createFileRoute } from '@tanstack/react-router'
import { WeeklyReviewList } from '@/components/weekly/weekly-review-list'
import { MainLayout } from '@/components/layout/main-layout'

export const Route = createFileRoute('/weekly')({
  component: WeeklyPage,
})

function WeeklyPage() {
  return (
    <MainLayout>
      <div className="space-y-6">
        <div className="space-y-2">
          <h1 className="text-2xl font-semibold">Weekly Review</h1>
          <p className="text-muted-foreground text-sm">
            Reflection snapshots. Data flows from the Tauri backend hooks.
          </p>
        </div>
        <WeeklyReviewList />
      </div>
    </MainLayout>
  )
}
