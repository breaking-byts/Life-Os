import { useMemo, useState } from 'react'
import { SearchIcon, Loader2Icon, PlusIcon } from 'lucide-react'
import { useExercises } from '@/hooks/useExercises'
import { tauri } from '@/lib/tauri'
import type { Exercise } from '@/types'

import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'

interface ExerciseSearchProps {
  onSelect: (exercise: Exercise) => void
  placeholder?: string
}

export function ExerciseSearch({
  onSelect,
  placeholder = 'Search exercises...',
}: ExerciseSearchProps) {
  const [query, setQuery] = useState('')
  const [open, setOpen] = useState(false)
  const [isCreating, setIsCreating] = useState(false)

  const normalizedQuery = useMemo(() => query.trim(), [query])
  const { data: exercises, isLoading } = useExercises(normalizedQuery)

  const handleSelect = (exercise: Exercise) => {
    onSelect(exercise)
    setQuery('')
    setOpen(false)
  }

  const handleCreateCustom = async () => {
    const name = normalizedQuery
    if (name.length < 2) return

    try {
      setIsCreating(true)
      const created = await tauri.createCustomExercise(name)
      handleSelect(created)
    } finally {
      setIsCreating(false)
    }
  }

  return (
    <div className="relative">
      <div className="relative">
        <SearchIcon className="absolute left-2 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <Input
          value={query}
          onChange={(e) => {
            setQuery(e.target.value)
            setOpen(e.target.value.length > 1)
          }}
          onFocus={() => query.length > 1 && setOpen(true)}
          placeholder={placeholder}
          className="pl-8"
        />
        {isLoading && (
          <Loader2Icon className="absolute right-2 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground animate-spin" />
        )}
      </div>

      {open && exercises && exercises.length > 0 && (
        <div className="absolute z-50 mt-1 w-full bg-popover border rounded-md shadow-lg">
          <ScrollArea className="max-h-64">
            <div className="p-1">
              {exercises.map((exercise) => (
                <button
                  key={exercise.id}
                  type="button"
                  className="w-full text-left px-3 py-2 text-sm hover:bg-accent rounded-sm flex flex-col gap-1"
                  onClick={() => handleSelect(exercise)}
                >
                  <span className="font-medium">{exercise.name}</span>
                  <div className="flex gap-1 flex-wrap">
                    {exercise.category && (
                      <Badge variant="outline" className="text-[10px] py-0">
                        {exercise.category}
                      </Badge>
                    )}
                    {exercise.muscles && (
                      <Badge variant="secondary" className="text-[10px] py-0">
                        {exercise.muscles}
                      </Badge>
                    )}
                  </div>
                </button>
              ))}
            </div>
          </ScrollArea>
        </div>
      )}

      {open &&
        normalizedQuery.length > 1 &&
        !isLoading &&
        exercises?.length === 0 && (
          <div className="absolute z-50 mt-1 w-full bg-popover border rounded-md shadow-lg">
            <button
              type="button"
              className="w-full text-left px-3 py-2 text-sm hover:bg-accent rounded-sm flex items-center gap-2"
              onClick={handleCreateCustom}
              disabled={isCreating}
            >
              {isCreating ? (
                <Loader2Icon className="h-4 w-4 animate-spin" />
              ) : (
                <PlusIcon className="h-4 w-4" />
              )}
              <span>Add “{normalizedQuery}”</span>
            </button>
          </div>
        )}
    </div>
  )
}
