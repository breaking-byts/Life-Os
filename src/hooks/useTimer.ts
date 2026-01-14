import { useState, useEffect, useCallback, useRef } from 'react'
import {
  loadPomodoroState,
  savePomodoroState,
  clearPomodoroRuntime,
  type PomodoroTimerMode,
  type PomodoroTimerState,
} from '@/lib/pomodoroPersistence'

type TimerState = PomodoroTimerState
type TimerMode = PomodoroTimerMode

interface UseTimerOptions {
  onComplete?: (completedMode: TimerMode) => void
  onWorkCompletedWhileAway?: () => void | Promise<void>
}

interface UseTimerReturn {
  state: TimerState
  mode: TimerMode
  timeLeft: number
  display: string
  progress: number
  cycles: number
  workMinutes: number
  breakMinutes: number
  start: () => void
  pause: () => void
  resume: () => void
  reset: () => void
  skip: () => void
  reconcile: () => void
  setWorkMinutes: (minutes: number) => void
  setBreakMinutes: (minutes: number) => void
}

function clampInt(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) return min
  return Math.min(max, Math.max(min, Math.round(value)))
}

function notify(mode: TimerMode) {
  if (!('Notification' in window)) return
  if (Notification.permission !== 'granted') return
  new Notification(
    mode === 'work' ? 'Work session complete!' : 'Break over, back to work!',
    {
      body: mode === 'work' ? 'Time for a break.' : 'Start your next session.',
    },
  )
}

export function useTimer({
  onComplete,
  onWorkCompletedWhileAway,
}: UseTimerOptions = {}): UseTimerReturn {
  const initial = loadPomodoroState()

  const [state, setState] = useState<TimerState>(initial.state)
  const [mode, setMode] = useState<TimerMode>(initial.mode)
  const [workMinutes, setWorkMinutesState] = useState(initial.workMinutes)
  const [breakMinutes, setBreakMinutesState] = useState(initial.breakMinutes)
  const [timeLeft, setTimeLeft] = useState(initial.timeLeft)
  const [cycles, setCycles] = useState(initial.cycles)

  const intervalRef = useRef<number | null>(null)
  const snapshotRef = useRef<number | null>(null)
  const completingRef = useRef(false)

  const totalSeconds = mode === 'work' ? workMinutes * 60 : breakMinutes * 60
  const progress =
    totalSeconds > 0 ? ((totalSeconds - timeLeft) / totalSeconds) * 100 : 0

  const snapshot = useCallback(
    (next?: Partial<ReturnType<typeof loadPomodoroState>>) => {
      const persisted = loadPomodoroState()
      savePomodoroState({
        ...persisted,
        version: 1,
        workMinutes,
        breakMinutes,
        mode,
        state,
        timeLeft,
        cycles,
        lastUpdatedAt: Date.now(),
        ...next, // Apply overrides AFTER defaults so they take precedence
      })
    },
    [workMinutes, breakMinutes, mode, state, timeLeft, cycles],
  )

  const clearTimer = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current)
      intervalRef.current = null
    }
  }, [])

  const clearSnapshot = useCallback(() => {
    if (snapshotRef.current) {
      clearInterval(snapshotRef.current)
      snapshotRef.current = null
    }
  }, [])

  const applyPhaseTransition = useCallback(
    async (completedMode: TimerMode) => {
      if (completingRef.current) return
      completingRef.current = true
      try {
        onComplete?.(completedMode)
        notify(completedMode)
      } finally {
        completingRef.current = false
      }
    },
    [onComplete],
  )

  const reconcile = useCallback(async () => {
    const persisted = loadPomodoroState()

    // Pull persisted durations/course even if state changed elsewhere
    setWorkMinutesState(persisted.workMinutes)
    setBreakMinutesState(persisted.breakMinutes)

    if (persisted.state !== 'running') {
      setState(persisted.state)
      setMode(persisted.mode)
      setTimeLeft(persisted.timeLeft)
      setCycles(persisted.cycles)
      return
    }

    const now = Date.now()
    const elapsedSeconds = Math.max(
      0,
      Math.floor((now - persisted.lastUpdatedAt) / 1000),
    )
    if (elapsedSeconds === 0) return

    let remaining = elapsedSeconds
    let nextMode: TimerMode = persisted.mode
    let nextTimeLeft = persisted.timeLeft
    let nextCycles = persisted.cycles

    let workCompletions = 0

    while (remaining > 0) {
      if (nextTimeLeft > remaining) {
        nextTimeLeft -= remaining
        remaining = 0
        break
      }

      // Phase completes
      remaining -= nextTimeLeft
      if (nextMode === 'work') {
        nextCycles += 1
        workCompletions += 1
        await applyPhaseTransition('work')
      } else {
        await applyPhaseTransition('break')
      }

      nextMode = nextMode === 'work' ? 'break' : 'work'
      const nextTotal =
        nextMode === 'work'
          ? persisted.workMinutes * 60
          : persisted.breakMinutes * 60
      nextTimeLeft = nextTotal

      // If we landed exactly on boundary, stop here
      if (remaining === 0) break
    }

    setMode(nextMode)
    setTimeLeft(nextTimeLeft)
    setCycles(nextCycles)
    setState('running')

    savePomodoroState({
      ...persisted,
      version: 1,
      mode: nextMode,
      state: 'running',
      timeLeft: nextTimeLeft,
      cycles: nextCycles,
      lastUpdatedAt: now,
    })

    if (workCompletions > 0) {
      await onWorkCompletedWhileAway?.()
    }
  }, [applyPhaseTransition, onWorkCompletedWhileAway])

  const start = useCallback(() => {
    setMode('work')
    setTimeLeft(workMinutes * 60)
    setState('running')
    savePomodoroState({
      ...loadPomodoroState(),
      version: 1,
      workMinutes,
      breakMinutes,
      mode: 'work',
      state: 'running',
      timeLeft: workMinutes * 60,
      lastUpdatedAt: Date.now(),
      cycles,
    })
  }, [workMinutes, breakMinutes, cycles])

  const pause = useCallback(() => {
    setState('paused')
    clearTimer()
    snapshot({ state: 'paused' })
  }, [clearTimer, snapshot])

  const resume = useCallback(() => {
    setState('running')
    snapshot({ state: 'running' })
  }, [snapshot])

  const reset = useCallback(() => {
    clearTimer()
    clearSnapshot()
    clearPomodoroRuntime()

    const persisted = loadPomodoroState()
    setState(persisted.state)
    setMode(persisted.mode)
    setTimeLeft(persisted.timeLeft)
    setCycles(persisted.cycles)
    setWorkMinutesState(persisted.workMinutes)
    setBreakMinutesState(persisted.breakMinutes)
  }, [clearTimer, clearSnapshot])

  const skip = useCallback(() => {
    clearTimer()

    const completedMode = mode
    const nextMode = mode === 'work' ? 'break' : 'work'
    const nextTimeLeft =
      nextMode === 'work' ? workMinutes * 60 : breakMinutes * 60

    setMode(nextMode)
    setTimeLeft(nextTimeLeft)

    if (completedMode === 'work') {
      setCycles((c) => c + 1)
    }

    setState('running')

    snapshot({
      mode: nextMode,
      state: 'running',
      timeLeft: nextTimeLeft,
      cycles: completedMode === 'work' ? cycles + 1 : cycles,
    })

    onComplete?.(completedMode)
  }, [
    mode,
    workMinutes,
    breakMinutes,
    clearTimer,
    snapshot,
    onComplete,
    cycles,
  ])

  const setWorkMinutes = useCallback(
    (minutes: number) => {
      const validMinutes = clampInt(minutes, 1, 120)
      setWorkMinutesState(validMinutes)

      const persisted = loadPomodoroState()
      const next: any = { workMinutes: validMinutes }

      if (state === 'idle' && mode === 'work') {
        setTimeLeft(validMinutes * 60)
        next.timeLeft = validMinutes * 60
      }

      savePomodoroState({
        ...persisted,
        ...next,
        version: 1,
        lastUpdatedAt: Date.now(),
      })
    },
    [state, mode],
  )

  const setBreakMinutes = useCallback(
    (minutes: number) => {
      const validMinutes = clampInt(minutes, 1, 60)
      setBreakMinutesState(validMinutes)

      const persisted = loadPomodoroState()
      const next: any = { breakMinutes: validMinutes }

      if (state === 'idle' && mode === 'break') {
        setTimeLeft(validMinutes * 60)
        next.timeLeft = validMinutes * 60
      }

      savePomodoroState({
        ...persisted,
        ...next,
        version: 1,
        lastUpdatedAt: Date.now(),
      })
    },
    [state, mode],
  )

  useEffect(() => {
    if (state !== 'running') return

    intervalRef.current = window.setInterval(() => {
      setTimeLeft((prev) => {
        if (prev <= 1) {
          const completedMode = mode
          const nextMode = mode === 'work' ? 'break' : 'work'
          const nextTimeLeft =
            nextMode === 'work' ? workMinutes * 60 : breakMinutes * 60

          if (completedMode === 'work') {
            setCycles((c) => c + 1)
          }

          setMode(nextMode)

          onComplete?.(completedMode)
          notify(completedMode)

          savePomodoroState({
            ...loadPomodoroState(),
            version: 1,
            workMinutes,
            breakMinutes,
            mode: nextMode,
            state: 'running',
            timeLeft: nextTimeLeft,
            cycles: completedMode === 'work' ? cycles + 1 : cycles,
            lastUpdatedAt: Date.now(),
          })

          return nextTimeLeft
        }

        // we update lastUpdatedAt via snapshot interval; keep tick lightweight
        return prev - 1
      })
    }, 1000)

    return () => clearTimer()
  }, [state, mode, workMinutes, breakMinutes, onComplete, clearTimer, cycles])

  useEffect(() => {
    if (state !== 'running') {
      clearSnapshot()
      return
    }

    snapshotRef.current = window.setInterval(() => {
      savePomodoroState({
        ...loadPomodoroState(),
        version: 1,
        workMinutes,
        breakMinutes,
        mode,
        state,
        timeLeft,
        cycles,
        lastUpdatedAt: Date.now(),
      })
    }, 8000)

    return () => clearSnapshot()
  }, [state, workMinutes, breakMinutes, mode, timeLeft, cycles, clearSnapshot])

  useEffect(() => {
    const onVisibility = () => {
      if (document.visibilityState === 'visible') {
        reconcile()
      }
    }

    window.addEventListener('focus', reconcile)
    document.addEventListener('visibilitychange', onVisibility)

    return () => {
      window.removeEventListener('focus', reconcile)
      document.removeEventListener('visibilitychange', onVisibility)
    }
  }, [reconcile])

  const minutes = Math.floor(timeLeft / 60)
  const seconds = timeLeft % 60
  const display = `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`

  return {
    state,
    mode,
    timeLeft,
    display,
    progress,
    cycles,
    workMinutes,
    breakMinutes,
    start,
    pause,
    resume,
    reset,
    skip,
    reconcile: () => void reconcile(),
    setWorkMinutes,
    setBreakMinutes,
  }
}
