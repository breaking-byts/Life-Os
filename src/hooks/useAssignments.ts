import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Assignment } from '@/types'
import { tauri } from '@/lib/tauri'

const ASSIGNMENTS_KEY = ['assignments']

export function useAssignments(courseId?: number) {
  const queryClient = useQueryClient()

  const assignmentsQuery = useQuery({
    queryKey: [ASSIGNMENTS_KEY, courseId],
    queryFn: () => tauri.getAssignments(courseId),
  })

  const createAssignment = useMutation({
    mutationFn: (data: Partial<Assignment>) => tauri.createAssignment(data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: [ASSIGNMENTS_KEY, courseId] }),
  })

  const updateAssignment = useMutation({
    mutationFn: ({ id, data }: { id: number; data: Partial<Assignment> }) =>
      tauri.updateAssignment(id, data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: [ASSIGNMENTS_KEY, courseId] }),
  })

  const toggleAssignment = useMutation({
    mutationFn: (id: number) => tauri.toggleAssignment(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: [ASSIGNMENTS_KEY, courseId] }),
  })

  const deleteAssignment = useMutation({
    mutationFn: (id: number) => tauri.deleteAssignment(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: [ASSIGNMENTS_KEY, courseId] }),
  })

  return {
    assignmentsQuery,
    createAssignment,
    updateAssignment,
    toggleAssignment,
    deleteAssignment,
  }
}
