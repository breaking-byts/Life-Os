import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Course } from '@/types'
import { tauri } from '@/lib/tauri'

const COURSES_KEY = ['courses']
const COURSES_WITH_PROGRESS_KEY = ['courses', 'with-progress']

export function useCourses() {
  const queryClient = useQueryClient()

  const coursesQuery = useQuery({
    queryKey: COURSES_KEY,
    queryFn: tauri.getCourses,
  })

  const createCourse = useMutation({
    mutationFn: (data: Partial<Course>) => tauri.createCourse(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: COURSES_KEY })
      queryClient.invalidateQueries({ queryKey: COURSES_WITH_PROGRESS_KEY })
    },
  })

  const updateCourse = useMutation({
    mutationFn: ({ id, data }: { id: number; data: Partial<Course> }) =>
      tauri.updateCourse(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: COURSES_KEY })
      queryClient.invalidateQueries({ queryKey: COURSES_WITH_PROGRESS_KEY })
    },
  })

  const deleteCourse = useMutation({
    mutationFn: (id: number) => tauri.deleteCourse(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: COURSES_KEY })
      queryClient.invalidateQueries({ queryKey: COURSES_WITH_PROGRESS_KEY })
    },
  })

  return { coursesQuery, createCourse, updateCourse, deleteCourse }
}

export function useCoursesWithProgress() {
  return useQuery({
    queryKey: COURSES_WITH_PROGRESS_KEY,
    queryFn: tauri.getCoursesWithProgress,
  })
}

export function useCourseAnalytics(courseId: number) {
  return useQuery({
    queryKey: ['courses', courseId, 'analytics'],
    queryFn: () => tauri.getCourseAnalytics(courseId),
    enabled: courseId > 0,
  })
}
