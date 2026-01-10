import { useState } from 'react'
import { PlusIcon } from 'lucide-react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { tauri } from '@/lib/tauri'

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
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'

interface WeeklyReviewFormProps {
    trigger?: React.ReactNode
}

function getWeekStart(): string {
    const now = new Date()
    const day = now.getDay()
    const diff = now.getDate() - day + (day === 0 ? -6 : 1) // Monday
    const monday = new Date(now.setDate(diff))
    return monday.toISOString().split('T')[0]
}

export function WeeklyReviewForm({ trigger }: WeeklyReviewFormProps) {
    const [open, setOpen] = useState(false)
    const queryClient = useQueryClient()

    const [wins, setWins] = useState('')
    const [improvements, setImprovements] = useState('')
    const [notes, setNotes] = useState('')

    const createReview = useMutation({
        mutationFn: (data: { week_start: string; wins?: string; improvements?: string; notes?: string }) =>
            tauri.createWeeklyReview(data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['weekly-reviews'] })
        },
    })

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()
        await createReview.mutateAsync({
            week_start: getWeekStart(),
            wins: wins || undefined,
            improvements: improvements || undefined,
            notes: notes || undefined,
        })
        setOpen(false)
        // Reset form
        setWins('')
        setImprovements('')
        setNotes('')
    }

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                {trigger ?? (
                    <Button size="sm">
                        <PlusIcon className="mr-1 h-4 w-4" />
                        New Review
                    </Button>
                )}
            </DialogTrigger>
            <DialogContent className="sm:max-w-lg">
                <form onSubmit={handleSubmit}>
                    <DialogHeader>
                        <DialogTitle>Weekly Review</DialogTitle>
                        <DialogDescription>
                            Reflect on your week. What went well? What could be better?
                        </DialogDescription>
                    </DialogHeader>

                    <div className="grid gap-4 py-4">
                        {/* Wins */}
                        <div className="grid gap-2">
                            <Label htmlFor="wins" className="flex items-center gap-2">
                                🎉 3 Wins This Week
                            </Label>
                            <Textarea
                                id="wins"
                                value={wins}
                                onChange={(e) => setWins(e.target.value)}
                                placeholder="1. Finished project ahead of deadline&#10;2. Maintained workout streak&#10;3. Got great feedback on presentation"
                                rows={4}
                            />
                        </div>

                        {/* Improvements */}
                        <div className="grid gap-2">
                            <Label htmlFor="improvements" className="flex items-center gap-2">
                                🎯 3 Focus Areas for Next Week
                            </Label>
                            <Textarea
                                id="improvements"
                                value={improvements}
                                onChange={(e) => setImprovements(e.target.value)}
                                placeholder="1. Start deep work before checking email&#10;2. Increase practice time by 30min&#10;3. Sleep before midnight"
                                rows={4}
                            />
                        </div>

                        {/* Notes */}
                        <div className="grid gap-2">
                            <Label htmlFor="notes">Additional Notes</Label>
                            <Textarea
                                id="notes"
                                value={notes}
                                onChange={(e) => setNotes(e.target.value)}
                                placeholder="Any other reflections, gratitude, or thoughts..."
                                rows={2}
                            />
                        </div>
                    </div>

                    <DialogFooter>
                        <Button type="submit" disabled={createReview.isPending}>
                            {createReview.isPending ? 'Saving...' : 'Save Review'}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    )
}
