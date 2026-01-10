import { useState, useEffect } from 'react'
import { PlusIcon, Pencil } from 'lucide-react'
import { useCourses } from '@/hooks/useCourses'
import type { Course } from '@/types'

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

interface CourseFormProps {
  course?: Course
  trigger?: React.ReactNode
}

const COLORS = [
  '#3b82f6', '#ef4444', '#10b981', '#f59e0b',
  '#8b5cf6', '#ec4899', '#06b6d4', '#84cc16',
]

export function CourseForm({ course, trigger }: CourseFormProps) {
  const [open, setOpen] = useState(false)
  const { createCourse, updateCourse } = useCourses()
  const isEditing = !!course

  const [name, setName] = useState('')
  const [code, setCode] = useState('')
  const [color, setColor] = useState(COLORS[0])
  const [creditHours, setCreditHours] = useState('3')
  const [targetHours, setTargetHours] = useState('6')

  useEffect(() => {
    if (course) {
      setName(course.name)
      setCode(course.code ?? '')
      setColor(course.color ?? COLORS[0])
      setCreditHours(String(course.credit_hours ?? 3))
      setTargetHours(String(course.target_weekly_hours ?? 6))
    } else {
      setName('')
      setCode('')
      setColor(COLORS[0])
      setCreditHours('3')
      setTargetHours('6')
    }
  }, [course, open])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    const data = {
      name,
      code: code || undefined,
      color,
      credit_hours: parseInt(creditHours) || 3,
      target_weekly_hours: parseFloat(targetHours) || 6,
    }

    if (isEditing && course) {
      await updateCourse.mutateAsync({ id: course.id, data })
    } else {
      await createCourse.mutateAsync(data)
    }
    setOpen(false)
  }

  const isPending = createCourse.isPending || updateCourse.isPending

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        {trigger ?? (
          <Button size="sm">
            <PlusIcon className="mr-1 h-4 w-4" />
            New Course
          </Button>
        )}
      </DialogTrigger>
      <DialogContent className="sm:max-w-md">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>{isEditing ? 'Edit Course' : 'New Course'}</DialogTitle>
            <DialogDescription>
              {isEditing ? 'Update course details.' : 'Add a new course to track.'}
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-4 py-4">
            <div className="grid gap-2">
              <Label htmlFor="name">Course Name *</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="e.g. Data Structures"
                required
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="grid gap-2">
                <Label htmlFor="code">Course Code</Label>
                <Input
                  id="code"
                  value={code}
                  onChange={(e) => setCode(e.target.value)}
                  placeholder="e.g. CS201"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="credits">Credits</Label>
                <Input
                  id="credits"
                  type="number"
                  min="1"
                  max="6"
                  value={creditHours}
                  onChange={(e) => setCreditHours(e.target.value)}
                />
              </div>
            </div>

            <div className="grid gap-2">
              <Label htmlFor="target">Weekly Target (hours)</Label>
              <Input
                id="target"
                type="number"
                min="1"
                max="40"
                step="0.5"
                value={targetHours}
                onChange={(e) => setTargetHours(e.target.value)}
              />
            </div>

            <div className="grid gap-2">
              <Label>Color</Label>
              <div className="flex gap-2">
                {COLORS.map((c) => (
                  <button
                    key={c}
                    type="button"
                    className={`h-6 w-6 rounded-full ring-offset-2 transition-all ${
                      color === c ? 'ring-2 ring-primary' : ''
                    }`}
                    style={{ backgroundColor: c }}
                    onClick={() => setColor(c)}
                  />
                ))}
              </div>
            </div>
          </div>

          <DialogFooter>
            <Button type="submit" disabled={!name || isPending}>
              {isPending ? 'Saving...' : isEditing ? 'Update' : 'Create'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export function EditCourseButton({ course }: { course: Course }) {
  return (
    <CourseForm
      course={course}
      trigger={
        <Button variant="ghost" size="icon-sm">
          <Pencil className="h-3 w-3" />
        </Button>
      }
    />
  )
}
