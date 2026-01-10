import { useState } from 'react'
import { SmileIcon } from 'lucide-react'
import { useCheckIn } from '@/hooks/useCheckIn'

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
import { Slider } from '@/components/ui/slider'
import { Textarea } from '@/components/ui/textarea'

interface CheckinFormProps {
    trigger?: React.ReactNode
}

const MOOD_LABELS = ['😢', '😕', '😐', '🙂', '😊', '😄', '🤩', '🔥', '✨', '🚀']
const ENERGY_LABELS = ['💤', '🔋', '⚡']

export function CheckinForm({ trigger }: CheckinFormProps) {
    const [open, setOpen] = useState(false)
    const { createCheckIn } = useCheckIn()

    const [mood, setMood] = useState(5)
    const [energy, setEnergy] = useState(5)
    const [notes, setNotes] = useState('')

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()
        await createCheckIn.mutateAsync({
            mood,
            energy,
            notes: notes || undefined,
        })
        setOpen(false)
        // Reset form
        setMood(5)
        setEnergy(5)
        setNotes('')
    }

    const getMoodEmoji = (value: number) => MOOD_LABELS[Math.min(value - 1, 9)]
    const getEnergyEmoji = (value: number) => {
        if (value <= 3) return ENERGY_LABELS[0]
        if (value <= 6) return ENERGY_LABELS[1]
        return ENERGY_LABELS[2]
    }

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                {trigger ?? (
                    <Button size="sm" variant="outline">
                        <SmileIcon className="mr-1 h-4 w-4" />
                        Check In
                    </Button>
                )}
            </DialogTrigger>
            <DialogContent className="sm:max-w-md">
                <form onSubmit={handleSubmit}>
                    <DialogHeader>
                        <DialogTitle>Daily Check-in</DialogTitle>
                        <DialogDescription>
                            How are you feeling today? Track your mood and energy.
                        </DialogDescription>
                    </DialogHeader>

                    <div className="grid gap-6 py-6">
                        {/* Mood Slider */}
                        <div className="grid gap-3">
                            <div className="flex items-center justify-between">
                                <Label>Mood</Label>
                                <span className="text-2xl">{getMoodEmoji(mood)}</span>
                            </div>
                            <div className="flex items-center gap-3">
                                <span className="text-muted-foreground text-sm">1</span>
                                <Slider
                                    value={[mood]}
                                    onValueChange={(v) => setMood(v[0])}
                                    min={1}
                                    max={10}
                                    step={1}
                                    className="flex-1"
                                />
                                <span className="text-muted-foreground text-sm">10</span>
                            </div>
                            <p className="text-muted-foreground text-center text-sm">
                                {mood}/10
                            </p>
                        </div>

                        {/* Energy Slider */}
                        <div className="grid gap-3">
                            <div className="flex items-center justify-between">
                                <Label>Energy</Label>
                                <span className="text-2xl">{getEnergyEmoji(energy)}</span>
                            </div>
                            <div className="flex items-center gap-3">
                                <span className="text-muted-foreground text-sm">1</span>
                                <Slider
                                    value={[energy]}
                                    onValueChange={(v) => setEnergy(v[0])}
                                    min={1}
                                    max={10}
                                    step={1}
                                    className="flex-1"
                                />
                                <span className="text-muted-foreground text-sm">10</span>
                            </div>
                            <p className="text-muted-foreground text-center text-sm">
                                {energy}/10
                            </p>
                        </div>

                        {/* Notes */}
                        <div className="grid gap-2">
                            <Label htmlFor="notes">Notes (optional)</Label>
                            <Textarea
                                id="notes"
                                value={notes}
                                onChange={(e) => setNotes(e.target.value)}
                                placeholder="How's your day going? Any thoughts..."
                                rows={3}
                            />
                        </div>
                    </div>

                    <DialogFooter>
                        <Button type="submit" disabled={createCheckIn.isPending}>
                            {createCheckIn.isPending ? 'Saving...' : 'Save Check-in'}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    )
}
