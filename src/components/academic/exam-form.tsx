import { useEffect, useState } from 'react'
import { Pencil, PlusIcon } from 'lucide-react'
import type { Exam } from '@/types'
import { useExams } from '@/hooks/useExams'
import { formatDate } from '@/lib/time'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'

interface ExamFormProps {
  courseId: number
  exam?: Exam
  trigger?: React.ReactNode
}

export function ExamForm({ courseId, exam, trigger }: ExamFormProps) {
  const [open, setOpen] = useState(false)
  const { createExam, updateExam } = useExams(courseId)
  const isEditing = !!exam

  const [title, setTitle] = useState('')
  const [examDate, setExamDate] = useState('')
  const [examTime, setExamTime] = useState('')
  const [location, setLocation] = useState('')
  const [durationMinutes, setDurationMinutes] = useState('')
  const [weight, setWeight] = useState('')
  const [notes, setNotes] = useState('')

  useEffect(() => {
    if (exam) {
      setTitle(exam.title)
      if (exam.exam_date) {
        setExamDate(formatDate(exam.exam_date, 'yyyy-MM-dd'))
        setExamTime(formatDate(exam.exam_date, 'HH:mm'))
      } else {
        setExamDate('')
        setExamTime('')
      }
      setLocation(exam.location ?? '')
      setDurationMinutes(exam.duration_minutes?.toString() ?? '')
      setWeight(exam.weight?.toString() ?? '')
      setNotes(exam.notes ?? '')
    } else {
      setTitle('')
      setExamDate('')
      setExamTime('')
      setLocation('')
      setDurationMinutes('')
      setWeight('')
      setNotes('')
    }
  }, [exam, open])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    let examDateTime: string | undefined
    if (examDate) {
      const dateStr = examTime ? `${examDate}T${examTime}` : `${examDate}T09:00`
      examDateTime = new Date(dateStr).toISOString()
    }

    const data = {
      course_id: courseId,
      title,
      exam_date: examDateTime,
      location: location || undefined,
      duration_minutes: durationMinutes ? parseInt(durationMinutes) : undefined,
      weight: weight ? parseFloat(weight) : undefined,
      notes: notes || undefined,
    }

    if (isEditing) {
      await updateExam.mutateAsync({ id: exam.id, data })
    } else {
      await createExam.mutateAsync(data)
    }
    setOpen(false)
  }

  const isPending = createExam.isPending || updateExam.isPending

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        {trigger ?? (
          <Button size="sm" variant="outline">
            <PlusIcon className="mr-1 h-4 w-4" />
            New Exam
          </Button>
        )}
      </DialogTrigger>
      <DialogContent className="sm:max-w-md">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>{isEditing ? 'Edit Exam' : 'New Exam'}</DialogTitle>
            <DialogDescription>
              {isEditing ? 'Update exam details.' : 'Add a new exam.'}
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-4 py-4">
            <div className="grid gap-2">
              <Label htmlFor="title">Title *</Label>
              <Input
                id="title"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="e.g. Midterm Exam"
                required
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="grid gap-2">
                <Label htmlFor="exam-date">Date</Label>
                <Input
                  id="exam-date"
                  type="date"
                  value={examDate}
                  onChange={(e) => setExamDate(e.target.value)}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="exam-time">Time</Label>
                <Input
                  id="exam-time"
                  type="time"
                  value={examTime}
                  onChange={(e) => setExamTime(e.target.value)}
                />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="grid gap-2">
                <Label htmlFor="location">Location</Label>
                <Input
                  id="location"
                  value={location}
                  onChange={(e) => setLocation(e.target.value)}
                  placeholder="e.g. Room 101"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="duration">Duration (min)</Label>
                <Input
                  id="duration"
                  type="number"
                  value={durationMinutes}
                  onChange={(e) => setDurationMinutes(e.target.value)}
                  placeholder="e.g. 120"
                />
              </div>
            </div>

            <div className="grid gap-2">
              <Label htmlFor="weight">Weight (%)</Label>
              <Input
                id="weight"
                type="number"
                step="0.1"
                value={weight}
                onChange={(e) => setWeight(e.target.value)}
                placeholder="e.g. 25 for 25% of final grade"
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="notes">Notes</Label>
              <Textarea
                id="notes"
                value={notes}
                onChange={(e) => setNotes(e.target.value)}
                placeholder="Topics covered, materials allowed, etc."
                rows={2}
              />
            </div>
          </div>

          <DialogFooter>
            <Button type="submit" disabled={!title || isPending}>
              {isPending ? 'Saving...' : isEditing ? 'Update' : 'Create'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export function EditExamButton({
  courseId,
  exam,
}: {
  courseId: number
  exam: Exam
}) {
  return (
    <ExamForm
      courseId={courseId}
      exam={exam}
      trigger={
        <Button variant="ghost" size="icon-sm">
          <Pencil className="h-3 w-3" />
        </Button>
      }
    />
  )
}
