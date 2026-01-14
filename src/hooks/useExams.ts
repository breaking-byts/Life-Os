import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Exam } from '@/types'
import { tauri } from '@/lib/tauri'

const EXAMS_KEY = ['exams']

export function useExams(courseId?: number) {
  const queryClient = useQueryClient()

  const examsQuery = useQuery({
    queryKey: [EXAMS_KEY, courseId],
    queryFn: () => tauri.getExams(courseId),
  })

  const createExam = useMutation({
    mutationFn: (data: Partial<Exam>) => tauri.createExam(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [EXAMS_KEY, courseId] })
      queryClient.invalidateQueries({ queryKey: ['exams', 'upcoming'] })
    },
  })

  const updateExam = useMutation({
    mutationFn: ({ id, data }: { id: number; data: Partial<Exam> }) =>
      tauri.updateExam(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [EXAMS_KEY, courseId] })
      queryClient.invalidateQueries({ queryKey: ['exams', 'upcoming'] })
    },
  })

  const deleteExam = useMutation({
    mutationFn: (id: number) => tauri.deleteExam(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [EXAMS_KEY, courseId] })
      queryClient.invalidateQueries({ queryKey: ['exams', 'upcoming'] })
    },
  })

  return {
    examsQuery,
    createExam,
    updateExam,
    deleteExam,
  }
}

export function useUpcomingExams(days: number = 14) {
  return useQuery({
    queryKey: ['exams', 'upcoming', days],
    queryFn: () => tauri.getUpcomingExams(days),
  })
}
