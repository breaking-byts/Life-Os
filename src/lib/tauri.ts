import { invoke as rawInvoke } from '@tauri-apps/api/core'
import type {
  Achievement,
  AgentRecommendation,
  AgentStatus,
  Assignment,
  BigThreeGoal,
  BigThreeInput,
  CalendarItem,
  CheckIn,
  Course,
  CourseAnalytics,
  CourseWithProgress,
  DetailedStats,
  Exam,
  Exercise,
  GoogleAccount,
  GoogleAuthBeginResponse,
  GoogleSyncStatus,
  PersonalRecord,
  PracticeLog,
  RichContext,
  Session,
  SimilarExperience,
  Skill,
  UserSettings,
  WeekPlanBlock,
  WeekPlanBlockInput,
  WeeklyReview,
  Workout,
  WorkoutExercise,
  WorkoutHeatmapDay,
  WorkoutTemplate,
  WorkoutTemplateExercise,
} from '@/types'

export type ApiErrorCode =
  | 'validation'
  | 'not_found'
  | 'conflict'
  | 'transient'
  | 'internal'

export type ApiError = {
  code: ApiErrorCode
  message: string
  details?: unknown
}

const DEFAULT_ERROR_MESSAGE = 'Something went wrong. Please try again.'

export function decodeApiError(error: unknown): ApiError {
  if (error && typeof error === 'object') {
    const candidate = error as { code?: unknown; message?: unknown; details?: unknown }
    if (typeof candidate.code === 'string' && typeof candidate.message === 'string') {
      return {
        code: candidate.code as ApiErrorCode,
        message: candidate.message,
        details: candidate.details,
      }
    }
  }

  if (error instanceof Error) {
    return {
      code: 'internal',
      message: error.message || DEFAULT_ERROR_MESSAGE,
    }
  }

  return {
    code: 'internal',
    message: DEFAULT_ERROR_MESSAGE,
  }
}

export function getApiErrorMessage(error: unknown): string {
  const decoded = decodeApiError(error)
  if (decoded.code === 'validation') {
    return decoded.message
  }

  if (decoded.code === 'not_found') {
    return decoded.message
  }

  if (decoded.code === 'conflict') {
    return decoded.message
  }

  if (decoded.code === 'transient') {
    return 'Temporary issue. Please try again.'
  }

  if (decoded.code === 'internal') {
    return DEFAULT_ERROR_MESSAGE
  }

  return decoded.message || DEFAULT_ERROR_MESSAGE
}

async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await rawInvoke<T>(command, args)
  } catch (error) {
    throw decodeApiError(error)
  }
}

export const tauri = {
  // Courses
  createCourse: (data: Partial<Course>) =>
    invoke<Course>('create_course', { data }),
  getCourses: () => invoke<Array<Course>>('get_courses'),
  getCourse: (id: number) => invoke<Course>('get_course', { id }),
  updateCourse: (id: number, data: Partial<Course>) =>
    invoke<Course>('update_course', { id, data }),
  deleteCourse: (id: number) => invoke<boolean>('delete_course', { id }),
  getCoursesWithProgress: () =>
    invoke<Array<CourseWithProgress>>('get_courses_with_progress'),
  getCourseAnalytics: (courseId: number) =>
    invoke<CourseAnalytics>('get_course_analytics', { courseId }),

  // Exams
  createExam: (data: Partial<Exam>) => invoke<Exam>('create_exam', { data }),
  getExams: (courseId?: number) =>
    invoke<Array<Exam>>('get_exams', { courseId }),
  getExam: (id: number) => invoke<Exam>('get_exam', { id }),
  updateExam: (id: number, data: Partial<Exam>) =>
    invoke<Exam>('update_exam', { id, data }),
  deleteExam: (id: number) => invoke<boolean>('delete_exam', { id }),
  getUpcomingExams: (days: number) =>
    invoke<Array<Exam>>('get_upcoming_exams', { days }),

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

  // Get exercises for a specific workout
  getWorkoutExercises: (workoutId: number) =>
    invoke<Array<WorkoutExercise>>('get_workout_exercises', { workoutId }),

  // Update workout
  updateWorkout: (id: number, data: Partial<Workout>) =>
    invoke<Workout>('update_workout', { id, data }),

  // Workout Templates
  getWorkoutTemplates: () => invoke<Array<WorkoutTemplate>>('get_workout_templates'),
  getTemplateExercises: (templateId: number) =>
    invoke<Array<WorkoutTemplateExercise>>('get_template_exercises', { templateId }),
  createWorkoutTemplate: (
    name: string,
    exercises: Array<Partial<WorkoutTemplateExercise>>,
  ) => invoke<WorkoutTemplate>('create_workout_template', { name, exercises }),
  updateWorkoutTemplate: (
    id: number,
    name: string,
    exercises: Array<Partial<WorkoutTemplateExercise>>,
  ) =>
    invoke<WorkoutTemplate>('update_workout_template', { id, name, exercises }),
  deleteWorkoutTemplate: (id: number) =>
    invoke<boolean>('delete_workout_template', { id }),

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
        confidence?: number
        insight_id?: number
        arm_name?: string
      }>
    >('get_insights'),

  // Agent learning
  recordInsightFeedback: (
    insightId: number,
    actedOn: boolean,
    feedbackScore?: number,
  ) =>
    invoke<void>('record_insight_feedback', {
      insightId,
      actedOn,
      feedbackScore,
    }),
  runPatternAnalysis: () => invoke<number>('run_pattern_analysis'),
  getUserProfile: () =>
    invoke<
      Array<{
        id: number
        dimension: string
        value_json: string
        confidence: number
        sample_count: number
        updated_at?: string
      }>
    >('get_user_profile'),

  // Intelligence Agent
  getAgentRecommendations: (count?: number) =>
    invoke<Array<AgentRecommendation>>('get_agent_recommendations', { count }),
  getAgentRecommendation: () =>
    invoke<AgentRecommendation>('get_agent_recommendation'),
  recordRecommendationFeedback: (
    recommendationId: number,
    accepted: boolean,
    alternativeChosen?: string,
    feedbackScore?: number,
    outcomeScore?: number,
  ) =>
    invoke<void>('record_recommendation_feedback', {
      recommendationId,
      accepted,
      alternativeChosen,
      feedbackScore,
      outcomeScore,
    }),
  recordActionCompleted: (
    actionType: string,
    description: string,
    outcomeScore: number,
    metadata?: Record<string, unknown>,
  ) =>
    invoke<number>('record_action_completed', {
      actionType,
      description,
      outcomeScore,
      metadata,
    }),
  getAgentStatus: () => invoke<AgentStatus>('get_agent_status'),
  getRichContext: () => invoke<RichContext>('get_rich_context'),

  // Big 3 Goals
  getBigThree: () => invoke<Array<BigThreeGoal>>('get_big_three'),
  setBigThree: (goals: Array<BigThreeInput>) =>
    invoke<void>('set_big_three', { goals }),
  completeBigThree: (goalId: number, satisfaction?: number) =>
    invoke<void>('complete_big_three', { goalId, satisfaction }),

  // Agent Maintenance
  runAgentMaintenance: () => invoke<void>('run_agent_maintenance'),
  getFeatureNames: () => invoke<Array<string>>('get_feature_names'),
  searchSimilarExperiences: (query: string, limit?: number) =>
    invoke<Array<SimilarExperience>>('search_similar_experiences', { query, limit }),
  setRewardWeights: (
    immediate: number,
    daily: number,
    weekly: number,
    monthly: number,
  ) =>
    invoke<void>('set_reward_weights', { immediate, daily, weekly, monthly }),
  setExplorationRate: (rate: number) =>
    invoke<void>('set_exploration_rate', { rate }),

  // Detailed Analytics (Dashboard Revamp)
  getDetailedStats: () => invoke<DetailedStats>('get_detailed_stats'),
  getUserSettings: () => invoke<UserSettings>('get_user_settings'),
  updateUserSettings: (
    weeklyWorkoutTarget: number,
    weeklyActiveSkillsTarget: number,
  ) =>
    invoke<UserSettings>('update_user_settings', {
      weeklyWorkoutTarget,
      weeklyActiveSkillsTarget,
    }),

  // Physical Analytics
  getWorkoutHeatmap: (months: number) =>
    invoke<Array<WorkoutHeatmapDay>>('get_workout_heatmap', { months }),
  getPersonalRecords: () => invoke<Array<PersonalRecord>>('get_personal_records'),
  checkAndUpdatePrs: (workoutId: number) =>
    invoke<Array<PersonalRecord>>('check_and_update_prs', { workoutId }),
  getAchievements: () => invoke<Array<Achievement>>('get_achievements'),
  checkAchievements: () => invoke<Array<Achievement>>('check_achievements'),

  // Calendar aggregation
  getCalendarItems: (
    startDate: string,
    endDate: string,
    includeAssignments?: boolean,
    includeExams?: boolean,
  ) =>
    invoke<Array<CalendarItem>>('get_calendar_items', {
      query: {
        startDate,
        endDate,
        includeAssignments,
        includeExams,
      },
    }),

  // Week plan blocks
  createWeekPlanBlock: (data: WeekPlanBlockInput) =>
    invoke<WeekPlanBlock>('create_week_plan_block', { data }),
  updateWeekPlanBlock: (id: number, data: WeekPlanBlockInput) =>
    invoke<WeekPlanBlock>('update_week_plan_block', { id, data }),
  acceptWeekPlanBlock: (id: number) =>
    invoke<WeekPlanBlock>('accept_week_plan_block', { id }),
  lockWeekPlanBlock: (id: number) =>
    invoke<WeekPlanBlock>('lock_week_plan_block', { id }),
  deleteWeekPlanBlock: (id: number) =>
    invoke<boolean>('delete_week_plan_block', { id }),
  clearSuggestedBlocks: (weekStartDate: string) =>
    invoke<number>('clear_suggested_blocks', { weekStartDate }),
  bulkCreatePlanBlocks: (blocks: Array<WeekPlanBlockInput>) =>
    invoke<Array<WeekPlanBlock>>('bulk_create_plan_blocks', { blocks }),

  // Google Calendar sync
  setGoogleClientId: (clientId: string) =>
    invoke<boolean>('set_google_client_id', { clientId }),
  googleOauthBegin: () =>
    invoke<GoogleAuthBeginResponse>('google_oauth_begin'),
  googleOauthComplete: (callbackUrl?: string) =>
    invoke<GoogleAccount>('google_oauth_complete', {
      callbackUrl,
    }),
  googleSyncNow: () => invoke<boolean>('google_sync_now'),
  getGoogleSyncStatus: () =>
    invoke<GoogleSyncStatus>('get_google_sync_status'),
  disconnectGoogle: () => invoke<boolean>('disconnect_google'),
}
