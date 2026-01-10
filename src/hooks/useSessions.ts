import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Session } from '@/types'
import { tauri } from '@/lib/tauri'

const SESSIONS_KEY = ['sessions']

export function useSessions(
  referenceId?: number,
  referenceType?: 'course' | 'skill',
) {
  const queryClient = useQueryClient()

  const sessionsQuery = useQuery({
    queryKey: [SESSIONS_KEY, referenceId, referenceType],
    queryFn: () => tauri.getSessions(referenceId, referenceType),
  })

  const startSession = useMutation({
    mutationFn: (data: Partial<Session>) => tauri.startSession(data),
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: [SESSIONS_KEY, referenceId, referenceType],
      }),
  })

  const endSession = useMutation({
    mutationFn: (id: number) => tauri.endSession(id),
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: [SESSIONS_KEY, referenceId, referenceType],
      }),
  })

  return { sessionsQuery, startSession, endSession }
}
