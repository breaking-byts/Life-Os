import {
  addDays,
  addWeeks,
  differenceInDays,
  differenceInHours,
  eachDayOfInterval,
  endOfMonth,
  endOfWeek,
  format,
  formatDistanceToNow,
  getDay,
  isPast,
  isThisWeek,
  isToday,
  isTomorrow,
  parseISO,
  startOfMonth,
  startOfWeek,
} from 'date-fns'

// Basic formatting
export const formatDate = (date?: string | number | Date, pattern = 'PPP') =>
  date ? format(new Date(date), pattern) : ''

export const fromNow = (date?: string | number | Date) =>
  date ? formatDistanceToNow(new Date(date), { addSuffix: true }) : ''

export const weekStart = (date: Date = new Date()) =>
  startOfWeek(date, { weekStartsOn: 1 })

export const weekEnd = (date: Date = new Date()) =>
  endOfWeek(date, { weekStartsOn: 1 })

export const monthStart = (date: Date = new Date()) => startOfMonth(date)

export const monthEnd = (date: Date = new Date()) => endOfMonth(date)

// Time awareness - current context
export function getCurrentTimeContext() {
  const now = new Date()
  const hour = now.getHours()

  let timeOfDay: 'morning' | 'afternoon' | 'evening' | 'night'
  if (hour >= 5 && hour < 12) {
    timeOfDay = 'morning'
  } else if (hour >= 12 && hour < 17) {
    timeOfDay = 'afternoon'
  } else if (hour >= 17 && hour < 21) {
    timeOfDay = 'evening'
  } else {
    timeOfDay = 'night'
  }

  return {
    now,
    hour,
    dayOfWeek: getDay(now),
    isWeekend: getDay(now) === 0 || getDay(now) === 6,
    timeOfDay,
    weekStart: weekStart(now),
    weekEnd: weekEnd(now),
    monthStart: monthStart(now),
    monthEnd: monthEnd(now),
  }
}

// Deadline utilities
export interface DeadlineInfo {
  date: Date
  daysUntil: number
  hoursUntil: number
  isOverdue: boolean
  isPast: boolean
  isToday: boolean
  isTomorrow: boolean
  isThisWeek: boolean
  urgency: 'overdue' | 'critical' | 'soon' | 'upcoming' | 'later'
  urgencyColor: string
  label: string
}

export function getDeadlineInfo(deadline?: string | Date): DeadlineInfo | null {
  if (!deadline) return null

  const date = typeof deadline === 'string' ? parseISO(deadline) : deadline
  const now = new Date()

  const daysUntil = differenceInDays(date, now)
  const hoursUntil = differenceInHours(date, now)
  const isOverdue = isPast(date)
  const todayCheck = isToday(date)
  const tomorrowCheck = isTomorrow(date)
  const thisWeekCheck = isThisWeek(date, { weekStartsOn: 1 })

  let urgency: DeadlineInfo['urgency']
  let urgencyColor: string
  let label: string

  if (isOverdue) {
    urgency = 'overdue'
    urgencyColor = 'text-red-500'
    label = `Overdue by ${Math.abs(daysUntil)} day${Math.abs(daysUntil) !== 1 ? 's' : ''}`
  } else if (todayCheck) {
    urgency = 'critical'
    urgencyColor = 'text-red-500'
    label = hoursUntil <= 0 ? 'Due now!' : `Due in ${hoursUntil}h`
  } else if (tomorrowCheck) {
    urgency = 'critical'
    urgencyColor = 'text-orange-500'
    label = 'Due tomorrow'
  } else if (daysUntil <= 3) {
    urgency = 'soon'
    urgencyColor = 'text-amber-500'
    label = `Due in ${daysUntil} days`
  } else if (thisWeekCheck) {
    urgency = 'upcoming'
    urgencyColor = 'text-blue-500'
    label = `Due ${format(date, 'EEEE')}`
  } else if (daysUntil <= 14) {
    urgency = 'upcoming'
    urgencyColor = 'text-muted-foreground'
    label = `Due ${format(date, 'MMM d')}`
  } else {
    urgency = 'later'
    urgencyColor = 'text-muted-foreground'
    label = format(date, 'MMM d, yyyy')
  }

  return {
    date,
    daysUntil,
    hoursUntil,
    isOverdue,
    isPast: isOverdue,
    isToday: todayCheck,
    isTomorrow: tomorrowCheck,
    isThisWeek: thisWeekCheck,
    urgency,
    urgencyColor,
    label,
  }
}

// Calendar utilities
export function getCalendarDays(
  year: number,
  month: number,
): Array<{ date: Date; isCurrentMonth: boolean }> {
  const start = startOfMonth(new Date(year, month))
  const end = endOfMonth(new Date(year, month))

  // Get the start of the week containing the first day of month
  const calendarStart = startOfWeek(start, { weekStartsOn: 1 })
  // Get the end of the week containing the last day of month
  const calendarEnd = endOfWeek(end, { weekStartsOn: 1 })

  const days = eachDayOfInterval({ start: calendarStart, end: calendarEnd })

  return days.map((date) => ({
    date,
    isCurrentMonth: date.getMonth() === month,
  }))
}

// Upcoming dates helper
export function getUpcomingDates(
  items: Array<{ date?: string | Date }>,
  daysAhead: number = 7,
): Map<string, typeof items> {
  const now = new Date()
  const endDate = addDays(now, daysAhead)
  const dateMap = new Map<string, typeof items>()

  const days = eachDayOfInterval({ start: now, end: endDate })
  for (const day of days) {
    dateMap.set(format(day, 'yyyy-MM-dd'), [])
  }

  for (const item of items) {
    if (!item.date) continue
    const itemDate =
      typeof item.date === 'string' ? parseISO(item.date) : item.date
    const key = format(itemDate, 'yyyy-MM-dd')
    if (dateMap.has(key)) {
      dateMap.get(key)!.push(item)
    }
  }

  return dateMap
}

// Format for display
export const formatDateTime = (date?: string | Date) =>
  date ? format(new Date(date), 'MMM d, yyyy h:mm a') : ''

export const formatTime = (date?: string | Date) =>
  date ? format(new Date(date), 'h:mm a') : ''

export const formatShortDate = (date?: string | Date) =>
  date ? format(new Date(date), 'MMM d') : ''

export const formatDayOfWeek = (date?: string | Date) =>
  date ? format(new Date(date), 'EEEE') : ''

// Week helpers
export function getWeekDays(date: Date = new Date()) {
  const start = weekStart(date)
  return eachDayOfInterval({ start, end: addDays(start, 6) })
}

export function getNextNWeeks(n: number = 4) {
  const weeks: Array<{ start: Date; end: Date }> = []
  let current = weekStart()

  for (let i = 0; i < n; i++) {
    weeks.push({
      start: current,
      end: addDays(current, 6),
    })
    current = addWeeks(current, 1)
  }

  return weeks
}

// Check if date is in range
export function isDateInRange(
  date: Date | string,
  start: Date,
  end: Date,
): boolean {
  const d = typeof date === 'string' ? parseISO(date) : date
  return d >= start && d <= end
}
