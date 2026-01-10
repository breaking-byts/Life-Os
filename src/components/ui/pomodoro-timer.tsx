import { useState, useEffect } from 'react'
import {
  PlayIcon,
  PauseIcon,
  RotateCcwIcon,
  SkipForwardIcon,
  SettingsIcon,
} from 'lucide-react'
import { useTimer } from '@/hooks/useTimer'
import { useCourses } from '@/hooks/useCourses'
import { useSessions } from '@/hooks/useSessions'
import {
  loadPomodoroState,
  savePomodoroState,
  clearPomodoroRuntime,
} from '@/lib/pomodoroPersistence'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover'

const TIMER_PRESETS = [
  { name: 'Classic', work: 25, break: 5 },
  { name: 'Short', work: 15, break: 3 },
  { name: 'Long', work: 50, break: 10 },
  { name: 'Deep Work', work: 90, break: 20 },
]

export function PomodoroTimer() {
  const initial = loadPomodoroState()
  const [selectedCourseId, setSelectedCourseId] = useState<string>(
    initial.selectedCourseId ?? 'none',
  )
  const [activeSessionId, setActiveSessionId] = useState<number | null>(
    initial.activeSessionId ?? null,
  )
  const [settingsOpen, setSettingsOpen] = useState(false)
  const { coursesQuery } = useCourses()
  const { startSession, endSession } = useSessions()

  const handleComplete = async (completedMode: 'work' | 'break') => {
    if (completedMode !== 'work') return
    if (!activeSessionId) return

    await endSession.mutateAsync(activeSessionId)
    setActiveSessionId(null)

    const persisted = loadPomodoroState()
    savePomodoroState({
      ...persisted,
      version: 1,
      activeSessionId: null,
      lastUpdatedAt: Date.now(),
    })
  }

  const handleWorkCompletedWhileAway = async () => {
    const persisted = loadPomodoroState()
    if (!persisted.activeSessionId) return

    await endSession.mutateAsync(persisted.activeSessionId)
    setActiveSessionId(null)

    savePomodoroState({
      ...persisted,
      version: 1,
      activeSessionId: null,
      lastUpdatedAt: Date.now(),
    })
  }

  const timer = useTimer({
    onComplete: handleComplete,
    onWorkCompletedWhileAway: handleWorkCompletedWhileAway,
  })

  const handleStart = async () => {
    if (timer.state === 'idle' && timer.mode === 'work') {
      const courseId =
        selectedCourseId && selectedCourseId !== 'none'
          ? Number(selectedCourseId)
          : undefined

      const session = await startSession.mutateAsync({
        session_type: 'study',
        reference_id: courseId,
        reference_type: courseId ? 'course' : undefined,
      })

      setActiveSessionId(session.id)
      const persisted = loadPomodoroState()
      savePomodoroState({
        ...persisted,
        version: 1,
        selectedCourseId,
        activeSessionId: session.id,
        lastUpdatedAt: Date.now(),
      })
    }

    timer.start()
  }

  const handleReset = async () => {
    if (activeSessionId) {
      await endSession.mutateAsync(activeSessionId)
      setActiveSessionId(null)
    }

    // runtime-only reset; keep durations and selected course
    clearPomodoroRuntime()
    timer.reset()

    const persisted = loadPomodoroState()
    savePomodoroState({
      ...persisted,
      version: 1,
      selectedCourseId,
      activeSessionId: null,
      lastUpdatedAt: Date.now(),
    })
  }

  const applyPreset = (preset: (typeof TIMER_PRESETS)[number]) => {
    timer.setWorkMinutes(preset.work)
    timer.setBreakMinutes(preset.break)

    const persisted = loadPomodoroState()
    savePomodoroState({
      ...persisted,
      version: 1,
      workMinutes: preset.work,
      breakMinutes: preset.break,
      lastUpdatedAt: Date.now(),
    })
  }

  // Request notification permission on mount
  useEffect(() => {
    if ('Notification' in window && Notification.permission === 'default') {
      Notification.requestPermission()
    }
  }, [])

  // Persist selected course
  useEffect(() => {
    const persisted = loadPomodoroState()
    if (persisted.selectedCourseId !== selectedCourseId) {
      savePomodoroState({
        ...persisted,
        version: 1,
        selectedCourseId,
        lastUpdatedAt: Date.now(),
      })
    }
  }, [selectedCourseId])

  const courses = coursesQuery.data ?? []
  const circumference = 2 * Math.PI * 45

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-base flex items-center justify-between">
          <span>Focus Timer</span>
          <div className="flex items-center gap-2">
            <span className="text-xs font-normal text-muted-foreground">
              {timer.cycles} {timer.cycles === 1 ? 'cycle' : 'cycles'}
            </span>
            <Popover open={settingsOpen} onOpenChange={setSettingsOpen}>
              <PopoverTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  disabled={timer.state !== 'idle'}
                  title="Timer Settings"
                >
                  <SettingsIcon className="h-3.5 w-3.5" />
                </Button>
              </PopoverTrigger>
              <PopoverContent className="w-72" align="end">
                <div className="space-y-4">
                  <div className="space-y-2">
                    <h4 className="font-medium text-sm">Timer Settings</h4>
                    <p className="text-xs text-muted-foreground">
                      Customize your focus and break durations.
                    </p>
                  </div>

                  <div className="space-y-3">
                    <div className="space-y-1.5">
                      <Label htmlFor="work-duration" className="text-xs">
                        Work Duration (minutes)
                      </Label>
                      <Input
                        id="work-duration"
                        type="number"
                        min={1}
                        max={120}
                        value={timer.workMinutes}
                        onChange={(e) =>
                          timer.setWorkMinutes(Number(e.target.value))
                        }
                        className="h-8"
                      />
                    </div>

                    <div className="space-y-1.5">
                      <Label htmlFor="break-duration" className="text-xs">
                        Break Duration (minutes)
                      </Label>
                      <Input
                        id="break-duration"
                        type="number"
                        min={1}
                        max={60}
                        value={timer.breakMinutes}
                        onChange={(e) =>
                          timer.setBreakMinutes(Number(e.target.value))
                        }
                        className="h-8"
                      />
                    </div>
                  </div>

                  <div className="space-y-2">
                    <Label className="text-xs">Quick Presets</Label>
                    <div className="grid grid-cols-2 gap-2">
                      {TIMER_PRESETS.map((preset) => (
                        <Button
                          key={preset.name}
                          variant="outline"
                          size="sm"
                          onClick={() => applyPreset(preset)}
                          className={
                            timer.workMinutes === preset.work &&
                            timer.breakMinutes === preset.break
                              ? 'border-primary'
                              : ''
                          }
                        >
                          <span className="text-xs">
                            {preset.name}
                            <span className="text-muted-foreground ml-1">
                              {preset.work}/{preset.break}
                            </span>
                          </span>
                        </Button>
                      ))}
                    </div>
                  </div>
                </div>
              </PopoverContent>
            </Popover>
          </div>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Timer circle */}
        <div className="flex justify-center">
          <div className="relative w-32 h-32">
            <svg className="w-full h-full -rotate-90">
              <circle
                cx="64"
                cy="64"
                r="45"
                fill="none"
                stroke="currentColor"
                strokeWidth="6"
                className="text-muted/30"
              />
              <circle
                cx="64"
                cy="64"
                r="45"
                fill="none"
                stroke="currentColor"
                strokeWidth="6"
                strokeLinecap="round"
                className={
                  timer.mode === 'work' ? 'text-primary' : 'text-green-500'
                }
                strokeDasharray={circumference}
                strokeDashoffset={circumference * (1 - timer.progress / 100)}
                style={{ transition: 'stroke-dashoffset 0.3s' }}
              />
            </svg>
            <div className="absolute inset-0 flex flex-col items-center justify-center">
              <span className="text-2xl font-mono font-semibold">
                {timer.display}
              </span>
              <span className="text-xs text-muted-foreground capitalize">
                {timer.mode}
              </span>
            </div>
          </div>
        </div>

        {/* Timer info */}
        {timer.state === 'idle' && (
          <div className="text-center text-xs text-muted-foreground">
            {timer.workMinutes}min work / {timer.breakMinutes}min break
          </div>
        )}

        {/* Course selector */}
        {timer.state === 'idle' && (
          <Select value={selectedCourseId} onValueChange={setSelectedCourseId}>
            <SelectTrigger className="w-full">
              <SelectValue placeholder="Select course (optional)" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="none">No course</SelectItem>
              {courses.map((course) => (
                <SelectItem key={course.id} value={String(course.id)}>
                  {course.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}

        {/* Controls */}
        <div className="flex justify-center gap-2">
          {timer.state === 'idle' && (
            <Button onClick={handleStart} className="flex-1">
              <PlayIcon className="h-4 w-4 mr-1" />
              Start
            </Button>
          )}

          {timer.state === 'running' && (
            <Button
              onClick={timer.pause}
              variant="secondary"
              className="flex-1"
            >
              <PauseIcon className="h-4 w-4 mr-1" />
              Pause
            </Button>
          )}

          {timer.state === 'paused' && (
            <Button onClick={timer.resume} className="flex-1">
              <PlayIcon className="h-4 w-4 mr-1" />
              Resume
            </Button>
          )}

          {timer.state !== 'idle' && (
            <>
              <Button onClick={timer.skip} variant="outline" size="icon">
                <SkipForwardIcon className="h-4 w-4" />
              </Button>
              <Button onClick={handleReset} variant="ghost" size="icon">
                <RotateCcwIcon className="h-4 w-4" />
              </Button>
            </>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
