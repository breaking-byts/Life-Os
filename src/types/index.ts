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
