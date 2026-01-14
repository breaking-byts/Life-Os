export interface User {
  id: number
  name: string
  email?: string
  created_at?: string
}

export interface Course {
  id: number
  user_id: number
  name: string
  code?: string
  color?: string
  credit_hours?: number
  target_weekly_hours?: number
  is_active?: number
  created_at?: string
  current_grade?: number
  target_grade?: number
}

export interface CourseWithProgress extends Course {
  hours_this_week: number
  weekly_percent: number
  total_hours: number
  upcoming_assignments: number
  overdue_assignments: number
}

export interface CourseAnalytics {
  course_id: number
  hours_this_week: number
  target_this_week: number
  weekly_percent: number
  total_hours: number
  sessions_count: number
  avg_session_duration: number
  weekly_history: WeeklyHours[]
}

export interface WeeklyHours {
  week_start: string
  hours: number
}

export interface Assignment {
  id: number
  course_id: number
  title: string
  description?: string
  due_date?: string
  priority?: string
  is_completed?: number
  completed_at?: string
  created_at?: string
}

export interface Session {
  id: number
  user_id: number
  session_type: 'study' | 'practice'
  reference_id?: number
  reference_type?: 'course' | 'skill'
  started_at: string
  ended_at?: string
  duration_minutes?: number
  notes?: string
}

export interface Skill {
  id: number
  user_id: number
  name: string
  category?: string
  description?: string
  target_weekly_hours?: number
  current_level?: number
  total_hours?: number
  created_at?: string
}

export interface PracticeLog {
  id: number
  skill_id: number
  duration_minutes: number
  notes?: string
  logged_at?: string
}

export interface Workout {
  id: number
  user_id: number
  name?: string
  duration_minutes?: number
  notes?: string
  logged_at?: string
}

export interface WorkoutExercise {
  id: number
  workout_id: number
  exercise_id?: number
  exercise_name: string
  sets?: number
  reps?: number
  weight?: number
  notes?: string
}

// Workout with exercises (for display)
export interface WorkoutWithExercises extends Workout {
  exercises: WorkoutExercise[]
}

// Template types
export interface WorkoutTemplate {
  id: number
  user_id: number
  name: string
  created_at?: string
  updated_at?: string
}

export interface WorkoutTemplateExercise {
  id: number
  template_id: number
  exercise_id?: number
  exercise_name: string
  default_sets?: number
  default_reps?: number
  default_weight?: number
  order_index?: number
}

export interface WorkoutTemplateWithExercises extends WorkoutTemplate {
  exercises: WorkoutTemplateExercise[]
}

export interface Exercise {
  id: number
  wger_id?: number
  name: string
  category?: string
  muscles?: string
  equipment?: string
  description?: string
  cached_at?: string
  source?: 'wger' | 'custom'
  created_at?: string
}

export interface CheckIn {
  id: number
  user_id: number
  mood?: number
  energy?: number
  notes?: string
  checked_in_at?: string
}

export interface WeeklyReview {
  id: number
  user_id: number
  week_start: string
  wins?: string
  improvements?: string
  notes?: string
  created_at?: string
}

// Intelligence Agent Types
export interface BanditAction {
  id: number
  name: string
  category: string
  description: string
  total_pulls: number
  total_reward: number
  is_enabled: boolean
}

export interface FeatureContribution {
  name: string
  value: number
  contribution: number
  direction: 'positive' | 'negative'
}

export interface PastExperience {
  description: string
  outcome: number
  similarity: number
  timestamp: string
}

export interface AlternativeAction {
  action: BanditAction
  expected_reward: number
  reason: string
}

export interface AgentRecommendation {
  action: BanditAction
  expected_reward: number
  uncertainty: number
  ucb_score: number
  explanation: string
  top_features: FeatureContribution[]
  similar_experiences: PastExperience[]
  alternatives: AlternativeAction[]
  confidence_level: 'low' | 'medium' | 'high'
  recommendation_id?: number
}

export interface AgentStatus {
  mode: string
  total_samples: number
  ready_for_neural: boolean
  memory_events: number
  avg_accuracy: number
  exploration_rate: number
  last_training?: string
}

export interface BigThreeGoal {
  id: number
  priority: number
  title: string
  description?: string
  category?: string
  is_completed: boolean
}

export interface BigThreeInput {
  title: string
  description?: string
  category?: string
}

export interface RichContext {
  hour_of_day: number
  day_of_week: number
  energy_level: number
  mood_level: number
  pomodoros_today: number
  assignment_urgency: number
  streak_days: number
  peak_focus_prob: number
  recovery_need: number
  [key: string]: number // Allow other features
}

export interface SimilarExperience {
  content: string
  event_type: string
  timestamp: string
  outcome?: number
  similarity: number
}

// Dashboard Revamp Types
export interface CourseProgress {
  course_id: number
  course_name: string
  code?: string
  color: string
  hours_this_week: number
  target_hours: number
  percent: number
  current_grade?: number
  target_grade?: number
}

export interface SkillProgress {
  skill_id: number
  skill_name: string
  category?: string
  hours_this_week: number
  target_weekly_hours: number
  weekly_percent: number
  total_hours: number
  target_hours: number
  mastery_percent: number
  current_level: number
}

export interface DetailedStats {
  study_hours_week: number
  study_target_week: number
  study_percent: number
  study_breakdown: CourseProgress[]
  practice_hours_week: number
  practice_target_week: number
  practice_percent: number
  practice_breakdown: SkillProgress[]
  workouts_week: number
  workout_target_week: number
  workout_percent: number
  active_skills_count: number
  skills_target: number
}

export interface UserSettings {
  weekly_workout_target: number
  weekly_active_skills_target: number
}

export interface WorkoutHeatmapDay {
  date: string
  count: number
  total_minutes: number
}

export interface Achievement {
  id: number
  achievement_type: string
  title: string
  description?: string
  category?: string
  achieved_at: string
  metadata?: string
}

export interface PersonalRecord {
  id: number
  exercise_name: string
  pr_type: string
  value: number
  achieved_at: string
  workout_id?: number
}

export interface Exam {
  id: number
  course_id: number
  title: string
  exam_date?: string
  location?: string
  duration_minutes?: number
  notes?: string
  grade?: number
  weight?: number
  created_at?: string
}
