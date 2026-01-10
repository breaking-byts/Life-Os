import { createFileRoute } from '@tanstack/react-router'
import { MainLayout } from '@/components/layout/main-layout'

export const Route = createFileRoute('/settings')({
  component: SettingsPage,
})

function SettingsPage() {
  return (
    <MainLayout>
      <div className="space-y-4">
        <div className="space-y-2">
          <h1 className="text-2xl font-semibold">Settings</h1>
          <p className="text-muted-foreground text-sm">
            App preferences will live here. Desktop shell uses Tauri v2.
          </p>
        </div>
        <div className="rounded-lg border p-4 text-sm text-muted-foreground">
          Placeholder settings panel. Add theme toggles, notifications, and data
          sync options in a future pass.
        </div>
      </div>
    </MainLayout>
  )
}
