import { useState, useEffect } from 'react'
import { PlusIcon, Pencil } from 'lucide-react'
import { useAssignments } from '@/hooks/useAssignments'
import type { Assignment } from '@/types'
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
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'

interface AssignmentFormProps {
    courseId: number
    assignment?: Assignment
    trigger?: React.ReactNode
}

export function AssignmentForm({ courseId, assignment, trigger }: AssignmentFormProps) {
    const [open, setOpen] = useState(false)
    const { createAssignment, updateAssignment } = useAssignments(courseId)
    const isEditing = !!assignment

    const [title, setTitle] = useState('')
    const [description, setDescription] = useState('')
    const [dueDate, setDueDate] = useState('')
    const [priority, setPriority] = useState<string>('medium')

    useEffect(() => {
        if (assignment) {
            setTitle(assignment.title)
            setDescription(assignment.description ?? '')
            setDueDate(assignment.due_date ? formatDate(assignment.due_date, 'yyyy-MM-dd') : '')
            setPriority(assignment.priority ?? 'medium')
        } else {
            setTitle('')
            setDescription('')
            setDueDate('')
            setPriority('medium')
        }
    }, [assignment, open])

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()
        const data = {
            course_id: courseId,
            title,
            description: description || undefined,
            due_date: dueDate ? new Date(dueDate).toISOString() : undefined,
            priority,
        }

        if (isEditing && assignment) {
            await updateAssignment.mutateAsync({ id: assignment.id, data })
        } else {
            await createAssignment.mutateAsync(data)
        }
        setOpen(false)
    }

    const isPending = createAssignment.isPending || updateAssignment.isPending

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                {trigger ?? (
                    <Button size="sm" variant="outline">
                        <PlusIcon className="mr-1 h-4 w-4" />
                        New Assignment
                    </Button>
                )}
            </DialogTrigger>
            <DialogContent className="sm:max-w-md">
                <form onSubmit={handleSubmit}>
                    <DialogHeader>
                        <DialogTitle>{isEditing ? 'Edit Assignment' : 'New Assignment'}</DialogTitle>
                        <DialogDescription>
                            {isEditing ? 'Update assignment details.' : 'Add a new assignment.'}
                        </DialogDescription>
                    </DialogHeader>

                    <div className="grid gap-4 py-4">
                        <div className="grid gap-2">
                            <Label htmlFor="title">Title *</Label>
                            <Input
                                id="title"
                                value={title}
                                onChange={(e) => setTitle(e.target.value)}
                                placeholder="e.g. Problem Set 3"
                                required
                            />
                        </div>

                        <div className="grid gap-2">
                            <Label htmlFor="description">Description</Label>
                            <Textarea
                                id="description"
                                value={description}
                                onChange={(e) => setDescription(e.target.value)}
                                placeholder="Optional notes..."
                                rows={2}
                            />
                        </div>

                        <div className="grid grid-cols-2 gap-4">
                            <div className="grid gap-2">
                                <Label htmlFor="due">Due Date</Label>
                                <Input
                                    id="due"
                                    type="date"
                                    value={dueDate}
                                    onChange={(e) => setDueDate(e.target.value)}
                                />
                            </div>
                            <div className="grid gap-2">
                                <Label>Priority</Label>
                                <Select value={priority} onValueChange={setPriority}>
                                    <SelectTrigger>
                                        <SelectValue />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="low">Low</SelectItem>
                                        <SelectItem value="medium">Medium</SelectItem>
                                        <SelectItem value="high">High</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>
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

export function EditAssignmentButton({ courseId, assignment }: { courseId: number; assignment: Assignment }) {
    return (
        <AssignmentForm
            courseId={courseId}
            assignment={assignment}
            trigger={
                <Button variant="ghost" size="icon-sm">
                    <Pencil className="h-3 w-3" />
                </Button>
            }
        />
    )
}
