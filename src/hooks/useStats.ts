import { useQuery } from '@tanstack/react-query'
import { tauri } from '@/lib/tauri'

export function useStats() {
  return useQuery({
    queryKey: ['stats'],
    queryFn: tauri.getStats,
  })
}
