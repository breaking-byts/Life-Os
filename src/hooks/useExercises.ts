import { useEffect, useMemo, useState } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import type { Exercise } from '@/types'
import { tauri } from '@/lib/tauri'

const EXERCISES_KEY = ['exercises-search']

function normalizeQuery(raw: string) {
  return raw.trim()
}

export function useExercises(query: string) {
  const queryClient = useQueryClient()
  const normalized = useMemo(() => normalizeQuery(query), [query])
  const [debounced, setDebounced] = useState(normalized)

  useEffect(() => {
    const handle = setTimeout(() => setDebounced(normalized), 150)
    return () => clearTimeout(handle)
  }, [normalized])

  const exercisesQuery = useQuery({
    queryKey: [...EXERCISES_KEY, debounced.toLowerCase()],
    enabled: debounced.length >= 2,
    queryFn: () => tauri.searchExercises(debounced),
    staleTime: 1000 * 60 * 60,
  })

  // If cache was empty, backend fetches asynchronously; do one retry soon.
  useEffect(() => {
    if (debounced.length < 2) return
    const handle = setTimeout(() => {
      queryClient.invalidateQueries({
        queryKey: [...EXERCISES_KEY, debounced.toLowerCase()],
      })
    }, 800)
    return () => clearTimeout(handle)
  }, [debounced, queryClient])

  return {
    data: exercisesQuery.data ?? ([] as Array<Exercise>),
    isLoading: exercisesQuery.isFetching,
  }
}
