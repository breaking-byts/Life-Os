export type PomodoroTimerMode = 'work' | 'break'
export type PomodoroTimerState = 'idle' | 'running' | 'paused'

const STORAGE_KEY = 'lifeos:pomodoro'
const STORAGE_VERSION = 1 as const

export interface PersistedPomodoroStateV1 {
  version: 1
  workMinutes: number
  breakMinutes: number
  mode: PomodoroTimerMode
  state: PomodoroTimerState
  timeLeft: number
  cycles: number
  selectedCourseId: string
  activeSessionId: number | null
  lastUpdatedAt: number
}

export type PersistedPomodoroState = PersistedPomodoroStateV1

export interface PomodoroRuntimeReset {
  mode: PomodoroTimerMode
  state: PomodoroTimerState
  timeLeft: number
  cycles: number
  activeSessionId: number | null
  lastUpdatedAt: number
}

export const pomodoroDefaults = {
  workMinutes: 25,
  breakMinutes: 5,
  selectedCourseId: 'none',
} as const

function clampInt(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) return min
  return Math.min(max, Math.max(min, Math.round(value)))
}

function safeParseJson(raw: string): unknown {
  try {
    return JSON.parse(raw)
  } catch {
    return null
  }
}

function normalizeV1(value: any): PersistedPomodoroStateV1 {
  const workMinutes = clampInt(
    value?.workMinutes ?? pomodoroDefaults.workMinutes,
    1,
    120,
  )
  const breakMinutes = clampInt(
    value?.breakMinutes ?? pomodoroDefaults.breakMinutes,
    1,
    60,
  )

  const mode: PomodoroTimerMode = value?.mode === 'break' ? 'break' : 'work'
  const state: PomodoroTimerState =
    value?.state === 'running' || value?.state === 'paused'
      ? value.state
      : 'idle'

  const selectedCourseId =
    typeof value?.selectedCourseId === 'string'
      ? value.selectedCourseId
      : 'none'

  const activeSessionId =
    typeof value?.activeSessionId === 'number' &&
    Number.isFinite(value.activeSessionId)
      ? value.activeSessionId
      : null

  const lastUpdatedAt =
    typeof value?.lastUpdatedAt === 'number' &&
    Number.isFinite(value.lastUpdatedAt)
      ? value.lastUpdatedAt
      : Date.now()

  const totalSeconds = mode === 'work' ? workMinutes * 60 : breakMinutes * 60
  const timeLeft = clampInt(value?.timeLeft ?? totalSeconds, 0, totalSeconds)

  const cycles = clampInt(value?.cycles ?? 0, 0, 1000000)

  return {
    version: 1,
    workMinutes,
    breakMinutes,
    mode,
    state,
    timeLeft,
    cycles,
    selectedCourseId,
    activeSessionId,
    lastUpdatedAt,
  }
}

export function loadPomodoroState(): PersistedPomodoroState {
  if (typeof window === 'undefined') {
    return {
      version: 1,
      workMinutes: pomodoroDefaults.workMinutes,
      breakMinutes: pomodoroDefaults.breakMinutes,
      mode: 'work',
      state: 'idle',
      timeLeft: pomodoroDefaults.workMinutes * 60,
      cycles: 0,
      selectedCourseId: pomodoroDefaults.selectedCourseId,
      activeSessionId: null,
      lastUpdatedAt: Date.now(),
    }
  }

  const raw = window.localStorage.getItem(STORAGE_KEY)
  if (!raw) {
    return normalizeV1({})
  }

  const parsed = safeParseJson(raw)
  if (!parsed || typeof parsed !== 'object') {
    return normalizeV1({})
  }

  const version = (parsed as any).version
  if (version !== STORAGE_VERSION) {
    return normalizeV1({})
  }

  return normalizeV1(parsed)
}

export function savePomodoroState(state: PersistedPomodoroState): void {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(state))
}

export function clearPomodoroState(): void {
  if (typeof window === 'undefined') return
  window.localStorage.removeItem(STORAGE_KEY)
}

export function clearPomodoroRuntime(): void {
  const current = loadPomodoroState()
  const reset: PomodoroRuntimeReset = {
    mode: 'work',
    state: 'idle',
    timeLeft: current.workMinutes * 60,
    cycles: 0,
    activeSessionId: null,
    lastUpdatedAt: Date.now(),
  }

  savePomodoroState({
    ...current,
    ...reset,
    version: 1,
  })
}
