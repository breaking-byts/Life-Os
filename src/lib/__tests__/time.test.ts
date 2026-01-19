/**
 * time.ts Library Tests - TDD Compliant
 * 
 * Testing date formatting, deadline calculations, and calendar utilities
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import {
    formatDate,
    fromNow,
    weekStart,
    weekEnd,
    getCurrentTimeContext,
    getDeadlineInfo,
    getCalendarDays,
    formatDateTime,
    formatTime,
    formatShortDate,
    getWeekDays,
    isDateInRange,
} from '../time'

describe('time utilities', () => {
    describe('formatDate', () => {
        it('formats date with default pattern PPP', () => {
            const result = formatDate('2026-01-19')
            expect(result).toContain('January')
            expect(result).toContain('19')
            expect(result).toContain('2026')
        })

        it('accepts custom format patterns', () => {
            const result = formatDate('2026-01-19', 'yyyy-MM-dd')
            expect(result).toBe('2026-01-19')
        })

        it('returns empty string for undefined', () => {
            expect(formatDate(undefined)).toBe('')
        })

        it('handles Date objects', () => {
            const date = new Date(2026, 0, 19)
            const result = formatDate(date, 'yyyy-MM-dd')
            expect(result).toBe('2026-01-19')
        })

        it('handles timestamps', () => {
            const timestamp = new Date(2026, 0, 19).getTime()
            const result = formatDate(timestamp, 'yyyy-MM-dd')
            expect(result).toBe('2026-01-19')
        })
    })

    describe('fromNow', () => {
        it('returns relative time description', () => {
            const futureDate = new Date(Date.now() + 1000 * 60 * 60 * 24)
            const result = fromNow(futureDate)
            expect(result).toContain('in')
        })

        it('returns empty string for undefined', () => {
            expect(fromNow(undefined)).toBe('')
        })
    })

    describe('weekStart / weekEnd', () => {
        it('returns Monday as week start', () => {
            // January 19, 2026 is a Monday
            const date = new Date(2026, 0, 21) // Wednesday
            const start = weekStart(date)
            expect(start.getDay()).toBe(1) // Monday
        })

        it('returns Sunday as week end', () => {
            const date = new Date(2026, 0, 21)
            const end = weekEnd(date)
            expect(end.getDay()).toBe(0) // Sunday
        })
    })

    describe('getCurrentTimeContext', () => {
        beforeEach(() => {
            vi.useFakeTimers()
        })

        afterEach(() => {
            vi.useRealTimers()
        })

        it('identifies morning (5-12)', () => {
            vi.setSystemTime(new Date(2026, 0, 19, 9, 0, 0))
            const ctx = getCurrentTimeContext()
            expect(ctx.timeOfDay).toBe('morning')
        })

        it('identifies afternoon (12-17)', () => {
            vi.setSystemTime(new Date(2026, 0, 19, 14, 0, 0))
            const ctx = getCurrentTimeContext()
            expect(ctx.timeOfDay).toBe('afternoon')
        })

        it('identifies evening (17-21)', () => {
            vi.setSystemTime(new Date(2026, 0, 19, 19, 0, 0))
            const ctx = getCurrentTimeContext()
            expect(ctx.timeOfDay).toBe('evening')
        })

        it('identifies night (21-5)', () => {
            vi.setSystemTime(new Date(2026, 0, 19, 23, 0, 0))
            const ctx = getCurrentTimeContext()
            expect(ctx.timeOfDay).toBe('night')
        })

        it('detects weekend correctly', () => {
            vi.setSystemTime(new Date(2026, 0, 17, 12, 0, 0)) // Saturday
            const ctx = getCurrentTimeContext()
            expect(ctx.isWeekend).toBe(true)
        })

        it('detects weekday correctly', () => {
            vi.setSystemTime(new Date(2026, 0, 19, 12, 0, 0)) // Monday
            const ctx = getCurrentTimeContext()
            expect(ctx.isWeekend).toBe(false)
        })
    })

    describe('getDeadlineInfo', () => {
        beforeEach(() => {
            vi.useFakeTimers()
            vi.setSystemTime(new Date(2026, 0, 19, 12, 0, 0))
        })

        afterEach(() => {
            vi.useRealTimers()
        })

        it('returns null for undefined deadline', () => {
            expect(getDeadlineInfo(undefined)).toBeNull()
        })

        it('identifies overdue deadlines', () => {
            const pastDate = new Date(2026, 0, 17)
            const info = getDeadlineInfo(pastDate)
            expect(info?.urgency).toBe('overdue')
            expect(info?.isOverdue).toBe(true)
        })

        it('identifies critical (today) deadlines', () => {
            const today = new Date(2026, 0, 19, 18, 0, 0)
            const info = getDeadlineInfo(today)
            expect(info?.urgency).toBe('critical')
            expect(info?.isToday).toBe(true)
        })

        it('identifies tomorrow deadlines', () => {
            const tomorrow = new Date(2026, 0, 20, 12, 0, 0)
            const info = getDeadlineInfo(tomorrow)
            expect(info?.urgency).toBe('critical')
            expect(info?.isTomorrow).toBe(true)
        })

        it('identifies soon (2-3 days) deadlines', () => {
            const soon = new Date(2026, 0, 21, 12, 0, 0)
            const info = getDeadlineInfo(soon)
            expect(info?.urgency).toBe('soon')
        })

        it('identifies later deadlines', () => {
            const later = new Date(2026, 2, 1)
            const info = getDeadlineInfo(later)
            expect(info?.urgency).toBe('later')
        })

        it('parses ISO string dates', () => {
            const info = getDeadlineInfo('2026-01-25T12:00:00')
            expect(info).not.toBeNull()
            expect(info?.daysUntil).toBeGreaterThan(0)
        })
    })

    describe('getCalendarDays', () => {
        it('returns array of calendar days for a month', () => {
            const days = getCalendarDays(2026, 0) // January 2026
            expect(days.length).toBeGreaterThanOrEqual(28)
            expect(days.length).toBeLessThanOrEqual(42)
        })

        it('includes previous month padding days', () => {
            const days = getCalendarDays(2026, 0)
            const hasNonCurrentMonth = days.some((d) => !d.isCurrentMonth)
            expect(hasNonCurrentMonth).toBe(true)
        })

        it('marks current month days correctly', () => {
            const days = getCalendarDays(2026, 0)
            const januaryDays = days.filter((d) => d.isCurrentMonth)
            expect(januaryDays.length).toBe(31)
        })
    })

    describe('formatting utilities', () => {
        it('formatDateTime includes date and time', () => {
            const result = formatDateTime('2026-01-19T14:30:00')
            expect(result).toContain('Jan')
            expect(result).toContain('19')
        })

        it('formatTime shows only time', () => {
            const result = formatTime('2026-01-19T14:30:00')
            expect(result).toMatch(/\d{1,2}:\d{2}\s*(AM|PM)/i)
        })

        it('formatShortDate shows abbreviated date', () => {
            const result = formatShortDate('2026-01-19')
            expect(result).toBe('Jan 19')
        })
    })

    describe('getWeekDays', () => {
        it('returns 7 days', () => {
            const days = getWeekDays(new Date(2026, 0, 19))
            expect(days).toHaveLength(7)
        })

        it('starts on Monday', () => {
            const days = getWeekDays(new Date(2026, 0, 21)) // Wednesday
            expect(days[0].getDay()).toBe(1) // Monday
        })
    })

    describe('isDateInRange', () => {
        it('returns true when date is in range', () => {
            const start = new Date(2026, 0, 1)
            const end = new Date(2026, 0, 31)
            const date = new Date(2026, 0, 15)
            expect(isDateInRange(date, start, end)).toBe(true)
        })

        it('returns false when date is before range', () => {
            const start = new Date(2026, 0, 10)
            const end = new Date(2026, 0, 20)
            const date = new Date(2026, 0, 5)
            expect(isDateInRange(date, start, end)).toBe(false)
        })

        it('returns false when date is after range', () => {
            const start = new Date(2026, 0, 10)
            const end = new Date(2026, 0, 20)
            const date = new Date(2026, 0, 25)
            expect(isDateInRange(date, start, end)).toBe(false)
        })

        it('returns true when date equals start', () => {
            const start = new Date(2026, 0, 10)
            const end = new Date(2026, 0, 20)
            expect(isDateInRange(start, start, end)).toBe(true)
        })

        it('accepts ISO string dates', () => {
            const start = new Date(2026, 0, 1)
            const end = new Date(2026, 0, 31)
            expect(isDateInRange('2026-01-15', start, end)).toBe(true)
        })
    })
})
