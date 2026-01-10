import { format, formatDistanceToNow, startOfWeek } from 'date-fns'

export const formatDate = (date?: string | number | Date, pattern = 'PPP') =>
  date ? format(new Date(date), pattern) : ''

export const fromNow = (date?: string | number | Date) =>
  date ? formatDistanceToNow(new Date(date), { addSuffix: true }) : ''

export const weekStart = (date: Date = new Date()) =>
  startOfWeek(date, { weekStartsOn: 1 })
