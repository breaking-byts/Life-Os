import { useState } from 'react'
import { PlusIcon } from 'lucide-react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useSkills } from '@/hooks/useSkills'
import { tauri } from '@/lib/tauri'
import type { Skill } from '@/types'

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

interface PracticeFormProps {
    skill?: Skill
    trigger?: React.ReactNode
}

export function PracticeForm({ skill, trigger }: PracticeFormProps) {
    const [open, setOpen] = useState(false)
    const { skillsQuery } = useSkills()
    const queryClient = useQueryClient()

    const [selectedSkillId, setSelectedSkillId] = useState(skill?.id?.toString() ?? '')
    const [duration, setDuration] = useState('30')
    const [notes, setNotes] = useState('')

    const logPractice = useMutation({
        mutationFn: (data: { skill_id: number; duration_minutes: number; notes?: string }) =>
            tauri.logPractice(data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['skills'] })
            queryClient.invalidateQueries({ queryKey: ['practice-logs'] })
        },
    })

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()
        if (!selectedSkillId) return

        await logPractice.mutateAsync({
            skill_id: Number(selectedSkillId),
            duration_minutes: parseInt(duration) || 30,
            notes: notes || undefined,
        })

        setDuration('30')
        setNotes('')
        setOpen(false)
    }

    const skills = skillsQuery.data ?? []

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                {trigger ?? (
                    <Button size="sm" variant="outline">
                        <PlusIcon className="mr-1 h-4 w-4" />
                        Log Practice
                    </Button>
                )}
            </DialogTrigger>
            <DialogContent className="sm:max-w-sm">
                <form onSubmit={handleSubmit}>
                    <DialogHeader>
                        <DialogTitle>Log Practice</DialogTitle>
                        <DialogDescription>
                            Record time spent practicing a skill.
                        </DialogDescription>
                    </DialogHeader>

                    <div className="grid gap-4 py-4">
                        {!skill && (
                            <div className="grid gap-2">
                                <Label>Skill *</Label>
                                <Select value={selectedSkillId} onValueChange={setSelectedSkillId}>
                                    <SelectTrigger>
                                        <SelectValue placeholder="Select skill" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {skills.map((s) => (
                                            <SelectItem key={s.id} value={String(s.id)}>
                                                {s.name}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                        )}

                        <div className="grid gap-2">
                            <Label htmlFor="duration">Duration (minutes) *</Label>
                            <Input
                                id="duration"
                                type="number"
                                min="1"
                                max="480"
                                value={duration}
                                onChange={(e) => setDuration(e.target.value)}
                                required
                            />
                        </div>

                        <div className="grid gap-2">
                            <Label htmlFor="notes">Notes</Label>
                            <Textarea
                                id="notes"
                                value={notes}
                                onChange={(e) => setNotes(e.target.value)}
                                placeholder="What did you work on?"
                                rows={2}
                            />
                        </div>
                    </div>

                    <DialogFooter>
                        <Button type="submit" disabled={!selectedSkillId || logPractice.isPending}>
                            {logPractice.isPending ? 'Saving...' : 'Log Practice'}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    )
}
