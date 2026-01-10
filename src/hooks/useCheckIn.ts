import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { CheckIn } from '@/types'
import { tauri } from '@/lib/tauri'

const CHECKIN_KEY = ['checkin']

export function useCheckIn() {
  const queryClient = useQueryClient()

  const checkInQuery = useQuery({
    queryKey: CHECKIN_KEY,
    queryFn: tauri.getTodayCheckIn,
  })

  const createCheckIn = useMutation({
    mutationFn: (data: Partial<CheckIn>) => tauri.createCheckIn(data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: CHECKIN_KEY }),
  })

  return { checkInQuery, createCheckIn }
}
