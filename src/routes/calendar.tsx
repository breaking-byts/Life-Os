import { createFileRoute } from '@tanstack/react-router'
import { useMemo, useRef, useState, type PointerEvent as ReactPointerEvent } from 'react'
import {
  addDays,
  addMinutes,
  addWeeks,
  differenceInMinutes,
  format,
  isSameDay,
  startOfDay,
} from 'date-fns'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { MainLayout } from '@/components/layout/main-layout'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import { formatTime, getWeekDays, weekStart } from '@/lib/time'
import { tauri } from '@/lib/tauri'
import type { CalendarItem, WeekPlanBlockInput } from '@/types'

export const Route = createFileRoute('/calendar')({
  component: CalendarPage,
})

const HOUR_HEIGHT = 56
const STEP_MINUTES = 15
const DEFAULT_BLOCK_MINUTES = 90

function CalendarPage() {
  const queryClient = useQueryClient()
  const [view, setView] = useState<'week' | 'day'>('week')
  const [anchorDate, setAnchorDate] = useState(new Date())
  const [selectedItem, setSelectedItem] = useState<CalendarItem | null>(null)

  const days = useMemo(() => {
    return view === 'week' ? getWeekDays(anchorDate) : [anchorDate]
  }, [anchorDate, view])

  const startDate = format(days[0], 'yyyy-MM-dd')
  const endDate = format(days[days.length - 1], 'yyyy-MM-dd')

  const calendarQuery = useQuery({
    queryKey: ['calendar-items', startDate, endDate],
    queryFn: () => tauri.getCalendarItems(startDate, endDate),
  })

  const syncStatusQuery = useQuery({
    queryKey: ['google-sync-status'],
    queryFn: tauri.getGoogleSyncStatus,
  })

  const syncNow = useMutation({
    mutationFn: tauri.googleSyncNow,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['calendar-items'] })
      queryClient.invalidateQueries({ queryKey: ['google-sync-status'] })
    },
  })

  const updateBlock = useMutation({
    mutationFn: ({ id, data }: { id: number; data: WeekPlanBlockInput }) =>
      tauri.updateWeekPlanBlock(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['calendar-items'] })
    },
  })

  const acceptBlock = useMutation({
    mutationFn: (id: number) => tauri.acceptWeekPlanBlock(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['calendar-items'] })
    },
  })

  const lockBlock = useMutation({
    mutationFn: (id: number) => tauri.lockWeekPlanBlock(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['calendar-items'] })
    },
  })

  const deleteBlock = useMutation({
    mutationFn: (id: number) => tauri.deleteWeekPlanBlock(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['calendar-items'] })
      setSelectedItem(null)
    },
  })

  const generatePlan = useMutation({
    mutationFn: async () => {
      const items = calendarQuery.data ?? []
      const weekStartDate = format(weekStart(days[0]), 'yyyy-MM-dd')
      await tauri.clearSuggestedBlocks(weekStartDate)
      const suggestions = buildSuggestedBlocks(days, items, weekStartDate)
      if (suggestions.length > 0) {
        await tauri.bulkCreatePlanBlocks(suggestions)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['calendar-items'] })
    },
  })

  const gridRef = useRef<HTMLDivElement | null>(null)
  const dragStateRef = useRef<DragState | null>(null)
  const dragPreviewRef = useRef<DragPreview | null>(null)
  const [dragPreview, setDragPreview] = useState<DragPreview | null>(null)

  const items = calendarQuery.data ?? []
  const dayCount = days.length

  const allDayByDay = useMemo(() => groupAllDayItems(days, items), [days, items])
  const timedByDay = useMemo(() => groupTimedItems(days, items), [days, items])

  const handlePrev = () => {
    setAnchorDate((prev) =>
      view === 'week' ? addWeeks(prev, -1) : addDays(prev, -1),
    )
  }

  const handleNext = () => {
    setAnchorDate((prev) =>
      view === 'week' ? addWeeks(prev, 1) : addDays(prev, 1),
    )
  }

  const handlePointerDown = (
    item: CalendarItem,
    event: ReactPointerEvent<HTMLDivElement>,
  ) => {
    if (item.source !== 'plan_block' || !item.editable) return
    const rect = event.currentTarget.getBoundingClientRect()
    const start = parseDate(item.start_at)
    const end = parseDate(item.end_at)
    const duration = Math.max(15, differenceInMinutes(end, start))
    const dayIndex = days.findIndex((day) => isSameDay(day, start))

    dragStateRef.current = {
      item,
      durationMinutes: duration,
      offsetY: event.clientY - rect.top,
      originalDayIndex: dayIndex === -1 ? 0 : dayIndex,
    }

    const preview = {
      start,
      end,
      dayIndex: dayIndex === -1 ? 0 : dayIndex,
    }
    dragPreviewRef.current = preview
    setDragPreview(preview)

    window.addEventListener('pointermove', handlePointerMove)
    window.addEventListener('pointerup', handlePointerUp)
  }

  const handlePointerMove = (event: PointerEvent) => {
    const drag = dragStateRef.current
    if (!drag || !gridRef.current) return

    const gridRect = gridRef.current.getBoundingClientRect()
    const rawY = event.clientY - gridRect.top - drag.offsetY

    const dayIndex = drag.originalDayIndex
    const minutesFromTop = clamp(
      Math.round((rawY / HOUR_HEIGHT) * 60 / STEP_MINUTES) * STEP_MINUTES,
      0,
      24 * 60 - drag.durationMinutes,
    )

    const dayStart = startOfDay(days[dayIndex])
    const newStart = addMinutes(dayStart, minutesFromTop)
    const newEnd = addMinutes(newStart, drag.durationMinutes)

    const preview: DragPreview = {
      start: newStart,
      end: newEnd,
      dayIndex,
    }

    dragPreviewRef.current = preview
    setDragPreview(preview)
  }

  const handlePointerUp = async () => {
    window.removeEventListener('pointermove', handlePointerMove)
    window.removeEventListener('pointerup', handlePointerUp)

    const drag = dragStateRef.current
    const preview = dragPreviewRef.current
    dragStateRef.current = null
    dragPreviewRef.current = null
    setDragPreview(null)

    if (!drag || !preview) return

    const planBlockId = parsePlanBlockId(drag.item.id)
    if (!planBlockId) return

    const newStart = preview.start
    const newEnd = preview.end
    const startAt = formatLocalDateTime(newStart)
    const endAt = formatLocalDateTime(newEnd)
    const weekStartDate = format(weekStart(newStart), 'yyyy-MM-dd')

    const payload: WeekPlanBlockInput = {
      week_start_date: weekStartDate,
      start_at: startAt,
      end_at: endAt,
      block_type: normalizeBlockType(drag.item.category),
      title: drag.item.title,
      status: drag.item.status ?? 'suggested',
    }

    await updateBlock.mutateAsync({ id: planBlockId, data: payload })

    if (
      (drag.item.status === 'accepted' || drag.item.status === 'locked') &&
      syncStatusQuery.data?.connected
    ) {
      syncNow.mutate()
    }
  }

  const handleAccept = async (item: CalendarItem) => {
    const id = parsePlanBlockId(item.id)
    if (!id) return
    await acceptBlock.mutateAsync(id)
    if (syncStatusQuery.data?.connected) {
      await syncNow.mutateAsync()
    }
  }

  const handleLock = async (item: CalendarItem) => {
    const id = parsePlanBlockId(item.id)
    if (!id) return
    await lockBlock.mutateAsync(id)
    if (syncStatusQuery.data?.connected) {
      await syncNow.mutateAsync()
    }
  }

  const handleDelete = async (item: CalendarItem) => {
    const id = parsePlanBlockId(item.id)
    if (!id) return
    await deleteBlock.mutateAsync(id)
  }

  return (
    <MainLayout>
      <div className="space-y-6">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div className="space-y-1">
            <h1 className="text-2xl font-semibold">Calendar</h1>
            <p className="text-sm text-muted-foreground">
              Syncs with Google Calendar and turns availability into plans.
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Button variant="outline" onClick={handlePrev}>
              Prev
            </Button>
            <Button variant="outline" onClick={handleNext}>
              Next
            </Button>
            <Button
              variant={view === 'week' ? 'default' : 'outline'}
              onClick={() => setView('week')}
            >
              Week
            </Button>
            <Button
              variant={view === 'day' ? 'default' : 'outline'}
              onClick={() => setView('day')}
            >
              Day
            </Button>
            <Button
              onClick={() => generatePlan.mutate()}
              disabled={generatePlan.isPending}
            >
              Generate plan
            </Button>
            <Button
              variant="outline"
              onClick={() => syncNow.mutate()}
              disabled={syncNow.isPending || !syncStatusQuery.data?.connected}
            >
              Sync now
            </Button>
          </div>
        </div>

        <div className="grid gap-6 lg:grid-cols-[1fr_320px]">
          <div className="space-y-4">
            <div className="rounded-xl border bg-card">
              <div
                className="grid border-b"
                style={{ gridTemplateColumns: `80px repeat(${dayCount}, minmax(0, 1fr))` }}
              >
                <div className="p-3 text-xs text-muted-foreground">Time</div>
                {days.map((day) => (
                  <div key={day.toISOString()} className="p-3">
                    <p className="text-sm font-semibold">
                      {format(day, 'EEE')}
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {format(day, 'MMM d')}
                    </p>
                  </div>
                ))}
              </div>
              <div
                className="grid border-b"
                style={{ gridTemplateColumns: `80px repeat(${dayCount}, minmax(0, 1fr))` }}
              >
                <div className="p-3 text-xs text-muted-foreground">All-day</div>
                {days.map((day) => {
                  const key = format(day, 'yyyy-MM-dd')
                  const allDayItems = allDayByDay.get(key) ?? []
                  return (
                    <div key={key} className="p-2">
                      <div className="space-y-1">
                        {allDayItems.length === 0 ? (
                          <p className="text-xs text-muted-foreground">—</p>
                        ) : (
                          allDayItems.map((item) => (
                            <div
                              key={item.id}
                              className={cn(
                                'text-xs rounded-md border px-2 py-1',
                                getItemClass(item),
                              )}
                              style={getItemStyle(item)}
                              onClick={() => setSelectedItem(item)}
                            >
                              {item.title}
                            </div>
                          ))
                        )}
                      </div>
                    </div>
                  )
                })}
              </div>
              <div
                className="grid"
                style={{ gridTemplateColumns: `80px 1fr` }}
              >
                <div className="relative" style={{ height: HOUR_HEIGHT * 24 }}>
                  {Array.from({ length: 24 }).map((_, hour) => (
                    <div
                      key={hour}
                      className="absolute left-2 text-xs text-muted-foreground"
                      style={{ top: hour * HOUR_HEIGHT - 6 }}
                    >
                      {format(new Date().setHours(hour, 0, 0, 0), 'ha')}
                    </div>
                  ))}
                </div>
                <div
                  ref={gridRef}
                  className="relative"
                  style={{ height: HOUR_HEIGHT * 24 }}
                >
                  <div
                    className="grid h-full"
                    style={{
                      gridTemplateColumns: `repeat(${dayCount}, minmax(0, 1fr))`,
                    }}
                  >
                    {days.map((day, dayIndex) => {
                      const dayKey = format(day, 'yyyy-MM-dd')
                      const itemsForDay = timedByDay.get(dayKey) ?? []
                      return (
                        <div
                          key={dayKey}
                          className="relative border-l border-muted/40"
                        >
                          {Array.from({ length: 24 }).map((_, hour) => (
                            <div
                              key={hour}
                              className="absolute left-0 right-0 border-t border-muted/30"
                              style={{ top: hour * HOUR_HEIGHT }}
                            />
                          ))}
                          {itemsForDay.map((item) => {
                            const preview =
                              dragPreview &&
                              dragStateRef.current?.item.id === item.id
                                ? dragPreview
                                : null

                            if (preview && preview.dayIndex !== dayIndex) {
                              return null
                            }

                            const start = preview
                              ? preview.start
                              : parseDate(item.start_at)
                            const end = preview
                              ? preview.end
                              : parseDate(item.end_at)

                            const minutesFromStart =
                              start.getHours() * 60 + start.getMinutes()
                            const duration = Math.max(
                              15,
                              differenceInMinutes(end, start),
                            )

                            const top =
                              (minutesFromStart / 60) * HOUR_HEIGHT
                            const height =
                              (duration / 60) * HOUR_HEIGHT

                            return (
                              <div
                                key={item.id}
                                className={cn(
                                  'absolute left-1 right-1 cursor-pointer select-none rounded-md border px-2 py-1 text-xs shadow-sm',
                                  getItemClass(item),
                                  preview && 'opacity-80',
                                )}
                                style={{ top, height, ...getItemStyle(item) }}
                                onClick={() => setSelectedItem(item)}
                                onPointerDown={(event) =>
                                  handlePointerDown(item, event)
                                }
                              >
                                <div className="font-semibold">
                                  {item.title}
                                </div>
                                <div className="text-[11px] opacity-80">
                                  {formatTime(start)} - {formatTime(end)}
                                </div>
                              </div>
                            )
                          })}
                        </div>
                      )
                    })}
                  </div>
                </div>
              </div>
            </div>
          </div>

          <aside className="space-y-4">
            <div className="rounded-xl border bg-card p-4">
              <div className="space-y-1">
                <p className="text-xs uppercase tracking-wide text-muted-foreground">
                  Google Sync
                </p>
                {syncStatusQuery.data?.connected ? (
                  <p className="text-sm font-semibold">
                    Connected {syncStatusQuery.data.email ? `as ${syncStatusQuery.data.email}` : ''}
                  </p>
                ) : (
                  <p className="text-sm font-semibold">Not connected</p>
                )}
                <p className="text-xs text-muted-foreground">
                  Last sync:{' '}
                  {syncStatusQuery.data?.last_sync
                    ? format(
                        new Date(syncStatusQuery.data.last_sync),
                        'MMM d, h:mm a',
                      )
                    : 'Never'}
                </p>
              </div>
              <div className="mt-3 flex flex-wrap gap-2">
                <Button
                  size="sm"
                  variant="outline"
                  disabled={!syncStatusQuery.data?.connected}
                  onClick={() => syncNow.mutate()}
                >
                  Sync now
                </Button>
              </div>
            </div>

            <div className="rounded-xl border bg-card p-4">
              <p className="text-xs uppercase tracking-wide text-muted-foreground">
                Details
              </p>
              {selectedItem ? (
                <div className="mt-3 space-y-3">
                  <div>
                    <p className="text-sm font-semibold">{selectedItem.title}</p>
                    <p className="text-xs text-muted-foreground">
                      {selectedItem.source.replace('_', ' ')}
                    </p>
                  </div>
                  <div className="text-xs text-muted-foreground">
                    {selectedItem.all_day
                      ? 'All day'
                      : `${formatTime(parseDate(selectedItem.start_at))} - ${formatTime(parseDate(selectedItem.end_at))}`}
                  </div>
                  {selectedItem.status && (
                    <div className="text-xs">
                      Status: {selectedItem.status}
                    </div>
                  )}
                  {selectedItem.source === 'plan_block' && (
                    <div className="flex flex-wrap gap-2">
                      {selectedItem.status === 'suggested' && (
                        <>
                          <Button
                            size="sm"
                            onClick={() => handleAccept(selectedItem)}
                          >
                            Accept
                          </Button>
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => handleLock(selectedItem)}
                          >
                            Lock
                          </Button>
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() => handleDelete(selectedItem)}
                          >
                            Dismiss
                          </Button>
                        </>
                      )}
                      {selectedItem.status === 'accepted' && (
                        <>
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => handleLock(selectedItem)}
                          >
                            Lock
                          </Button>
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() => handleDelete(selectedItem)}
                          >
                            Remove
                          </Button>
                        </>
                      )}
                    </div>
                  )}
                </div>
              ) : (
                <p className="mt-3 text-sm text-muted-foreground">
                  Select an item to see details.
                </p>
              )}
            </div>
          </aside>
        </div>
      </div>
    </MainLayout>
  )
}

interface DragState {
  item: CalendarItem
  durationMinutes: number
  offsetY: number
  originalDayIndex: number
}

interface DragPreview {
  start: Date
  end: Date
  dayIndex: number
}

function parseDate(value: string) {
  if (!value) return new Date()
  if (value.length === 10) return new Date(`${value}T00:00:00`)
  return new Date(value)
}

function formatLocalDateTime(date: Date) {
  return format(date, "yyyy-MM-dd'T'HH:mm:ss")
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max)
}

function parsePlanBlockId(id: string) {
  if (!id.startsWith('wpb_')) return null
  const value = Number(id.replace('wpb_', ''))
  return Number.isFinite(value) ? value : null
}

function normalizeBlockType(type?: string | null) {
  if (!type) return 'study'
  const allowed = new Set(['study', 'assignment', 'exam_prep', 'break', 'weekly_task'])
  return allowed.has(type) ? type : 'study'
}

function getItemClass(item: CalendarItem) {
  if (item.source === 'course_meeting') {
    return 'bg-sky-500/15 text-sky-100 border-sky-400/40'
  }
  if (item.source === 'calendar_event') {
    return 'bg-slate-500/15 text-slate-100 border-slate-400/40'
  }
  if (item.source === 'assignment') {
    return 'bg-amber-500/15 text-amber-100 border-amber-400/40'
  }
  if (item.source === 'exam') {
    return 'bg-rose-500/15 text-rose-100 border-rose-400/40'
  }
  if (item.source === 'plan_block') {
    if (item.status === 'suggested') {
      return 'bg-emerald-500/15 text-emerald-100 border-emerald-400/40'
    }
    if (item.status === 'locked') {
      return 'bg-emerald-700/20 text-emerald-100 border-emerald-500/50'
    }
    return 'bg-emerald-500/25 text-emerald-100 border-emerald-400/50'
  }
  return 'bg-muted text-foreground border-muted'
}

function getItemStyle(item: CalendarItem) {
  if (!item.color) return undefined
  return {
    borderLeftColor: item.color,
    borderLeftWidth: '3px',
  }
}

function groupAllDayItems(days: Date[], items: Array<CalendarItem>) {
  const map = new Map<string, Array<CalendarItem>>()
  for (const day of days) {
    map.set(format(day, 'yyyy-MM-dd'), [])
  }

  items
    .filter((item) => item.all_day)
    .forEach((item) => {
      const start = parseDate(item.start_at)
      const key = format(start, 'yyyy-MM-dd')
      if (!map.has(key)) {
        map.set(key, [])
      }
      map.get(key)!.push(item)
    })

  return map
}

function groupTimedItems(days: Date[], items: Array<CalendarItem>) {
  const map = new Map<string, Array<CalendarItem>>()
  for (const day of days) {
    map.set(format(day, 'yyyy-MM-dd'), [])
  }

  items
    .filter((item) => !item.all_day)
    .forEach((item) => {
      const start = parseDate(item.start_at)
      const key = format(start, 'yyyy-MM-dd')
      if (!map.has(key)) {
        map.set(key, [])
      }
      map.get(key)!.push(item)
    })

  return map
}

function buildSuggestedBlocks(
  days: Date[],
  items: Array<CalendarItem>,
  weekStartDate: string,
): Array<WeekPlanBlockInput> {
  const busyMap = new Map<string, Array<{ start: Date; end: Date }>>()
  for (const day of days) {
    busyMap.set(format(day, 'yyyy-MM-dd'), [])
  }

  for (const item of items) {
    if (item.source === 'plan_block' && item.status === 'suggested') {
      continue
    }
    const start = parseDate(item.start_at)
    const end = parseDate(item.end_at)
    const key = format(start, 'yyyy-MM-dd')
    if (!busyMap.has(key)) {
      busyMap.set(key, [])
    }
    if (item.all_day) {
      const dayStart = startOfDay(start)
      const blockStart = addMinutes(dayStart, 8 * 60)
      const blockEnd = addMinutes(dayStart, 20 * 60)
      busyMap.get(key)!.push({ start: blockStart, end: blockEnd })
    } else {
      busyMap.get(key)!.push({ start, end })
    }
  }

  const suggestions: Array<WeekPlanBlockInput> = []

  for (const day of days) {
    const key = format(day, 'yyyy-MM-dd')
    const intervals = busyMap.get(key) ?? []
    const slot = findFirstSlot(day, intervals, DEFAULT_BLOCK_MINUTES)
    if (!slot) continue

    suggestions.push({
      week_start_date: weekStartDate,
      start_at: formatLocalDateTime(slot.start),
      end_at: formatLocalDateTime(slot.end),
      block_type: 'study',
      title: 'Focus block',
      status: 'suggested',
    })
  }

  return suggestions
}

function findFirstSlot(
  day: Date,
  busy: Array<{ start: Date; end: Date }>,
  durationMinutes: number,
) {
  const startMinutes = 8 * 60
  const endMinutes = 20 * 60
  const dayStart = startOfDay(day)

  for (
    let minutes = startMinutes;
    minutes + durationMinutes <= endMinutes;
    minutes += STEP_MINUTES
  ) {
    const slotStart = addMinutes(dayStart, minutes)
    const slotEnd = addMinutes(slotStart, durationMinutes)

    const overlaps = busy.some(
      (interval) =>
        slotStart < interval.end && interval.start < slotEnd,
    )

    if (!overlaps) {
      return { start: slotStart, end: slotEnd }
    }
  }

  return null
}
