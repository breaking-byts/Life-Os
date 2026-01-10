import { createFileRoute } from '@tanstack/react-router'
import { SkillList } from '@/components/skills/skill-list'
import { MainLayout } from '@/components/layout/main-layout'

export const Route = createFileRoute('/skills')({
  component: SkillsPage,
})

function SkillsPage() {
  return (
    <MainLayout>
      <div className="space-y-6">
        <div className="space-y-2">
          <h1 className="text-2xl font-semibold">Skills</h1>
          <p className="text-muted-foreground text-sm">
            Manage skills and practice logs. Hooks call the Tauri commands.
          </p>
        </div>
        <SkillList />
      </div>
    </MainLayout>
  )
}
