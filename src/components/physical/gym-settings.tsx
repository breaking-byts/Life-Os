import { useEffect, useState } from 'react'
import { SettingsIcon } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useUpdateUserSettings, useUserSettings } from '@/hooks/useStats'

export function GymSettings() {
  const { data: settings } = useUserSettings()
  const updateSettings = useUpdateUserSettings()
  const [open, setOpen] = useState(false)
  const [workoutTarget, setWorkoutTarget] = useState('4')

  useEffect(() => {
    if (settings) {
      setWorkoutTarget(String(settings.weekly_workout_target ?? 4))
    }
  }, [settings])

  const handleSave = () => {
    updateSettings.mutate(
      {
        weeklyWorkoutTarget: parseInt(workoutTarget),
        weeklyActiveSkillsTarget: settings?.weekly_active_skills_target ?? 5,
      },
      {
        onSuccess: () => setOpen(false),
      },
    )
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="outline" size="sm">
          <SettingsIcon className="h-4 w-4 mr-1" />
          Set Target
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Gym Settings</DialogTitle>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label htmlFor="workout-target">Weekly Workout Target</Label>
            <Select value={workoutTarget} onValueChange={setWorkoutTarget}>
              <SelectTrigger id="workout-target" className="w-full">
                <SelectValue placeholder="Select target" />
              </SelectTrigger>
              <SelectContent>
                {[1, 2, 3, 4, 5, 6, 7].map((num) => (
                  <SelectItem key={num} value={String(num)}>
                    {num} workout{num > 1 ? 's' : ''} per week
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">
              How many workouts do you want to complete each week?
            </p>
          </div>
        </div>
        <div className="flex justify-end gap-2">
          <Button variant="outline" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={updateSettings.isPending}>
            {updateSettings.isPending ? 'Saving...' : 'Save'}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
