import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Course } from '@/types'
import { tauri } from '@/lib/tauri'

const COURSES_KEY = ['courses']

export function useCourses() {
  const queryClient = useQueryClient()

  const coursesQuery = useQuery({
    queryKey: COURSES_KEY,
    queryFn: tauri.getCourses,
  })

  const createCourse = useMutation({
    mutationFn: (data: Partial<Course>) => tauri.createCourse(data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: COURSES_KEY }),
  })

  const updateCourse = useMutation({
    mutationFn: ({ id, data }: { id: number; data: Partial<Course> }) =>
      tauri.updateCourse(id, data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: COURSES_KEY }),
  })

  const deleteCourse = useMutation({
    mutationFn: (id: number) => tauri.deleteCourse(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: COURSES_KEY }),
  })

  return { coursesQuery, createCourse, updateCourse, deleteCourse }
}
