import { invoke } from '@tauri-apps/api/core'
import type {
  Assignment,
  CheckIn,
  Course,
  Exercise,
  PracticeLog,
  Session,
  Skill,
  WeeklyReview,
  Workout,
  WorkoutExercise,
} from '@/types'

export const tauri = {
  // Courses
  createCourse: (data: Partial<Course>) =>
    invoke<Course>('create_course', { data }),
  getCourses: () => invoke<Array<Course>>('get_courses'),
  getCourse: (id: number) => invoke<Course>('get_course', { id }),
  updateCourse: (id: number, data: Partial<Course>) =>
    invoke<Course>('update_course', { id, data }),
  deleteCourse: (id: number) => invoke<boolean>('delete_course', { id }),

  // Assignments
  createAssignment: (data: Partial<Assignment>) =>
    invoke<Assignment>('create_assignment', { data }),
  getAssignments: (courseId?: number) =>
    invoke<Array<Assignment>>('get_assignments', { courseId }),
  updateAssignment: (id: number, data: Partial<Assignment>) =>
    invoke<Assignment>('update_assignment', { id, data }),
  deleteAssignment: (id: number) =>
    invoke<boolean>('delete_assignment', { id }),
  toggleAssignment: (id: number) =>
    invoke<Assignment>('toggle_assignment', { id }),

  // Sessions
  startSession: (data: Partial<Session>) =>
    invoke<Session>('start_session', { data }),
  endSession: (id: number) => invoke<Session>('end_session', { id }),
  getSessions: (referenceId?: number, referenceType?: string) =>
    invoke<Array<Session>>('get_sessions', { referenceId, referenceType }),

  // Skills
  createSkill: (data: Partial<Skill>) =>
    invoke<Skill>('create_skill', { data }),
  getSkills: () => invoke<Array<Skill>>('get_skills'),
  updateSkill: (id: number, data: Partial<Skill>) =>
    invoke<Skill>('update_skill', { id, data }),
  deleteSkill: (id: number) => invoke<boolean>('delete_skill', { id }),

  logPractice: (data: Partial<PracticeLog>) =>
    invoke<PracticeLog>('log_practice', { data }),
  getPracticeLogs: (skillId?: number) =>
    invoke<Array<PracticeLog>>('get_practice_logs', { skillId }),

  // Workouts
  createWorkout: (data: Partial<Workout>) =>
    invoke<Workout>('create_workout', { data }),
  getWorkouts: () => invoke<Array<Workout>>('get_workouts'),
  getWorkout: (id: number) => invoke<Workout>('get_workout', { id }),
  deleteWorkout: (id: number) => invoke<boolean>('delete_workout', { id }),

  addExerciseToWorkout: (data: Partial<WorkoutExercise>) =>
    invoke<WorkoutExercise>('add_exercise_to_workout', { data }),
  updateWorkoutExercise: (id: number, data: Partial<WorkoutExercise>) =>
    invoke<WorkoutExercise>('update_workout_exercise', { id, data }),
  removeExercise: (id: number) => invoke<boolean>('remove_exercise', { id }),

  // Exercises search
  fetchAndCacheExercises: () => invoke<number>('fetch_and_cache_exercises'),
  searchExercises: (query: string) =>
    invoke<Array<Exercise>>('search_exercises', { query }),
  createCustomExercise: (name: string) =>
    invoke<Exercise>('create_custom_exercise', { name }),

  // Check-ins
  createCheckIn: (data: Partial<CheckIn>) =>
    invoke<CheckIn>('create_checkin', { data }),
  getTodayCheckIn: () => invoke<CheckIn | null>('get_today_checkin'),
  getCheckIns: () => invoke<Array<CheckIn>>('get_checkins'),

  // Weekly review
  createWeeklyReview: (data: Partial<WeeklyReview>) =>
    invoke<WeeklyReview>('create_weekly_review', { data }),
  getWeeklyReviews: () => invoke<Array<WeeklyReview>>('get_weekly_reviews'),

  // Debug
  getExerciseCacheStats: () =>
    invoke<{
      count: number
      sample: Array<{ id: number; name: string; source: string }>
    }>('get_exercise_cache_stats'),

  // Analytics
  getStats: () =>
    invoke<{
      study_hours_week: number
      practice_hours_week: number
      workouts_week: number
      active_streaks: number
    }>('get_stats'),
  getStreaks: () =>
    invoke<{
      study_streak: number
      workout_streak: number
      practice_streak: number
      checkin_streak: number
    }>('get_streaks'),
  getInsights: () =>
    invoke<
      Array<{
        icon: string
        message: string
        category: string
      }>
    >('get_insights'),
}
