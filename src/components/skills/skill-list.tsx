import { Trash2Icon, ClockIcon } from 'lucide-react'
import { useSkills } from '@/hooks/useSkills'
import { fromNow } from '@/lib/time'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { SkillForm, EditSkillButton } from './skill-form'
import { PracticeForm } from './practice-form'

export function SkillList() {
  const { skillsQuery, deleteSkill } = useSkills()
  const skills = skillsQuery.data ?? []

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Skills</CardTitle>
        <div className="flex gap-2">
          <PracticeForm />
          <SkillForm />
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        {skills.length === 0 && (
          <p className="text-muted-foreground text-sm">No skills yet. Add one to start tracking!</p>
        )}
        {skills.map((skill) => {
          const progress = Math.min(100, ((skill.total_hours ?? 0) / 100) * 100)
          return (
            <div
              key={skill.id}
              className="border-border bg-muted/60 rounded-md border p-3 space-y-2"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div>
                    <p className="font-medium">{skill.name}</p>
                    <p className="text-muted-foreground text-xs">
                      Added {fromNow(skill.created_at)}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-1">
                  {skill.category && <Badge variant="outline">{skill.category}</Badge>}
                  <PracticeForm
                    skill={skill}
                    trigger={
                      <Button variant="ghost" size="icon-sm">
                        <ClockIcon className="h-3 w-3" />
                      </Button>
                    }
                  />
                  <EditSkillButton skill={skill} />
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    onClick={() => deleteSkill.mutate(skill.id)}
                  >
                    <Trash2Icon className="h-3 w-3 text-destructive" />
                  </Button>
                </div>
              </div>
              <div className="flex items-center gap-2 text-xs">
                <span className="text-muted-foreground">{skill.total_hours?.toFixed(1) ?? 0}h total</span>
                <Progress value={progress} className="flex-1 h-1.5" />
                <span>Lvl {skill.current_level ?? 1}</span>
              </div>
            </div>
          )
        })}
      </CardContent>
    </Card>
  )
}

