import { useEffect, useState } from 'react'
import { Pencil, PlusIcon } from 'lucide-react'
import type { Skill } from '@/types'
import { useSkills } from '@/hooks/useSkills'

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

interface SkillFormProps {
  skill?: Skill
  trigger?: React.ReactNode
}

const CATEGORIES = [
  'Music',
  'Art',
  'Programming',
  'Languages',
  'Sports',
  'Writing',
  'Other',
]

export function SkillForm({ skill, trigger }: SkillFormProps) {
  const [open, setOpen] = useState(false)
  const { createSkill, updateSkill } = useSkills()
  const isEditing = !!skill

  const [name, setName] = useState('')
  const [category, setCategory] = useState('')
  const [description, setDescription] = useState('')
  const [targetHours, setTargetHours] = useState('5')

  useEffect(() => {
    if (skill) {
      setName(skill.name)
      setCategory(skill.category ?? '')
      setDescription(skill.description ?? '')
      setTargetHours(String(skill.target_weekly_hours ?? 5))
    } else {
      setName('')
      setCategory('')
      setDescription('')
      setTargetHours('5')
    }
  }, [skill, open])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    const data = {
      name,
      category: category || undefined,
      description: description || undefined,
      target_weekly_hours: parseFloat(targetHours) || 5,
    }

    if (isEditing) {
      await updateSkill.mutateAsync({ id: skill.id, data })
    } else {
      await createSkill.mutateAsync(data)
    }
    setOpen(false)
  }

  const isPending = createSkill.isPending || updateSkill.isPending

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        {trigger ?? (
          <Button size="sm">
            <PlusIcon className="mr-1 h-4 w-4" />
            New Skill
          </Button>
        )}
      </DialogTrigger>
      <DialogContent className="sm:max-w-md">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>{isEditing ? 'Edit Skill' : 'New Skill'}</DialogTitle>
            <DialogDescription>
              {isEditing
                ? 'Update skill details.'
                : 'Add a new skill to track.'}
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-4 py-4">
            <div className="grid gap-2">
              <Label htmlFor="name">Skill Name *</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="e.g. Guitar, Spanish, Drawing"
                required
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="grid gap-2">
                <Label htmlFor="category">Category</Label>
                <Input
                  id="category"
                  value={category}
                  onChange={(e) => setCategory(e.target.value)}
                  placeholder="e.g. Music"
                  list="categories"
                />
                <datalist id="categories">
                  {CATEGORIES.map((c) => (
                    <option key={c} value={c} />
                  ))}
                </datalist>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="target">Weekly Target (hrs)</Label>
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
            </div>

            <div className="grid gap-2">
              <Label htmlFor="description">Notes</Label>
              <Textarea
                id="description"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Optional description or goals..."
                rows={2}
              />
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

export function EditSkillButton({ skill }: { skill: Skill }) {
  return (
    <SkillForm
      skill={skill}
      trigger={
        <Button variant="ghost" size="icon-sm">
          <Pencil className="h-3 w-3" />
        </Button>
      }
    />
  )
}
