import { useState } from 'react'
import { ChevronDownIcon, PlusIcon } from 'lucide-react'
import type { WorkoutTemplate, WorkoutTemplateExercise } from '@/types'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useTemplateExercises, useWorkoutTemplates } from '@/hooks/useWorkouts'

interface ExerciseEntry {
  exercise_name: string
  exercise_id?: number
  sets?: string
  reps?: string
  weight?: string
}

interface TemplateSelectorProps {
  onSelectTemplate: (exercises: Array<ExerciseEntry>) => void
  disabled?: boolean
}

export function TemplateSelector({
  onSelectTemplate,
  disabled,
}: TemplateSelectorProps) {
  const { templatesQuery } = useWorkoutTemplates()
  const [selectedTemplateId, setSelectedTemplateId] = useState<number | null>(
    null,
  )

  const templates = templatesQuery.data ?? []

  const handleSelectTemplate = (template: WorkoutTemplate) => {
    setSelectedTemplateId(template.id)
  }

  // When we have a selected template, fetch its exercises
  const { data: templateExercises } = useTemplateExercises(
    selectedTemplateId ?? 0,
  )

  // Effect to load exercises when template exercises are loaded
  if (selectedTemplateId && templateExercises && templateExercises.length > 0) {
    const exercises: Array<ExerciseEntry> = templateExercises.map(
      (ex: WorkoutTemplateExercise) => ({
        exercise_name: ex.exercise_name,
        exercise_id: ex.exercise_id ?? undefined,
        sets: ex.default_sets?.toString() ?? '',
        reps: ex.default_reps?.toString() ?? '',
        weight: ex.default_weight?.toString() ?? '',
      }),
    )
    onSelectTemplate(exercises)
    setSelectedTemplateId(null) // Reset after loading
  }

  if (templates.length === 0) {
    return null
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" size="sm" disabled={disabled}>
          <PlusIcon className="h-3 w-3 mr-1" />
          Load Template
          <ChevronDownIcon className="h-3 w-3 ml-1" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-48">
        {templates.map((template) => (
          <DropdownMenuItem
            key={template.id}
            onClick={() => handleSelectTemplate(template)}
          >
            {template.name}
          </DropdownMenuItem>
        ))}
        {templates.length === 0 && (
          <>
            <DropdownMenuSeparator />
            <DropdownMenuItem disabled>No templates saved yet</DropdownMenuItem>
          </>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
