import { useState } from 'react'
import { CheckCircle2Icon, CircleIcon, PlusIcon, StarIcon } from 'lucide-react'

import type { BigThreeGoal, BigThreeInput } from '@/types'
import { useBigThree } from '@/hooks/useIntelligence'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Slider } from '@/components/ui/slider'

const CATEGORIES = [
  { value: 'productivity', label: 'Productivity', icon: '💻' },
  { value: 'physical', label: 'Physical', icon: '💪' },
  { value: 'learning', label: 'Learning', icon: '📚' },
  { value: 'personal', label: 'Personal', icon: '🎯' },
  { value: 'social', label: 'Social', icon: '👥' },
]

function GoalItem({
  goal,
  onComplete,
  isLoading,
}: {
  goal: BigThreeGoal
  onComplete: (satisfaction: number) => void
  isLoading: boolean
}) {
  const [showSatisfaction, setShowSatisfaction] = useState(false)
  const [satisfaction, setSatisfaction] = useState([7])

  const handleComplete = () => {
    if (goal.is_completed) return
    setShowSatisfaction(true)
  }

  const confirmComplete = () => {
    onComplete(satisfaction[0])
    setShowSatisfaction(false)
  }

  const categoryInfo = CATEGORIES.find((c) => c.value === goal.category)

  return (
    <div
      className={`flex items-center gap-3 rounded-lg border p-3 transition-all ${
        goal.is_completed
          ? 'border-green-500/30 bg-green-500/5'
          : 'border-border bg-muted/40 hover:bg-muted/60'
      }`}
    >
      <button
        onClick={handleComplete}
        disabled={goal.is_completed || isLoading}
        className="shrink-0"
      >
        {goal.is_completed ? (
          <CheckCircle2Icon className="h-5 w-5 text-green-500" />
        ) : (
          <CircleIcon className="h-5 w-5 text-muted-foreground hover:text-primary transition-colors" />
        )}
      </button>

      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-muted-foreground font-mono text-sm">
            {goal.priority}.
          </span>
          <span
            className={`font-medium ${goal.is_completed ? 'line-through text-muted-foreground' : ''}`}
          >
            {goal.title}
          </span>
        </div>
        {goal.description && (
          <p className="text-xs text-muted-foreground mt-0.5 truncate">
            {goal.description}
          </p>
        )}
      </div>

      {categoryInfo && (
        <Badge variant="outline" className="shrink-0 text-xs">
          {categoryInfo.icon} {categoryInfo.label}
        </Badge>
      )}

      {/* Satisfaction dialog */}
      {showSatisfaction && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-background rounded-lg p-6 w-80 space-y-4">
            <h3 className="font-medium">How satisfied are you?</h3>
            <div className="space-y-2">
              <Slider
                value={satisfaction}
                onValueChange={setSatisfaction}
                min={1}
                max={10}
                step={1}
              />
              <div className="flex justify-between text-xs text-muted-foreground">
                <span>Not satisfied</span>
                <span className="font-medium">{satisfaction[0]}/10</span>
                <span>Very satisfied</span>
              </div>
            </div>
            <div className="flex gap-2 justify-end">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowSatisfaction(false)}
              >
                Cancel
              </Button>
              <Button size="sm" onClick={confirmComplete}>
                Complete
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

function AddGoalDialog({
  onAdd,
  existingCount,
}: {
  onAdd: (goals: Array<BigThreeInput>) => void
  existingCount: number
}) {
  const [open, setOpen] = useState(false)
  const [title, setTitle] = useState('')
  const [description, setDescription] = useState('')
  const [category, setCategory] = useState<string>('')

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!title.trim()) return

    onAdd([
      {
        title: title.trim(),
        description: description.trim() || undefined,
        category: category || undefined,
      },
    ])
    setTitle('')
    setDescription('')
    setCategory('')
    setOpen(false)
  }

  if (existingCount >= 3) return null

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="outline" size="sm" className="w-full">
          <PlusIcon className="h-4 w-4 mr-2" />
          Add Goal ({3 - existingCount} remaining)
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Add Big 3 Goal</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">
              What&apos;s your goal?
            </label>
            <Input
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="e.g., Complete math assignment"
              autoFocus
            />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">Details (optional)</label>
            <Input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="e.g., Chapter 5 problems 1-20"
            />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">Category</label>
            <Select value={category} onValueChange={setCategory}>
              <SelectTrigger>
                <SelectValue placeholder="Select a category" />
              </SelectTrigger>
              <SelectContent>
                {CATEGORIES.map((cat) => (
                  <SelectItem key={cat.value} value={cat.value}>
                    {cat.icon} {cat.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="flex gap-2 justify-end">
            <Button
              type="button"
              variant="outline"
              onClick={() => setOpen(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={!title.trim()}>
              Add Goal
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export function AgentBigThree() {
  const { goals, isLoading, setGoals, completeGoal } = useBigThree()

  const completedCount = goals.filter((g) => g.is_completed).length
  const progress = goals.length > 0 ? (completedCount / goals.length) * 100 : 0

  const handleAddGoal = (newGoals: Array<BigThreeInput>) => {
    // Append new goals to existing ones
    const existingGoals = goals.map((g) => ({
      title: g.title,
      description: g.description,
      category: g.category,
    }))
    setGoals.mutate([...existingGoals, ...newGoals])
  }

  const handleComplete = (goalId: number, satisfaction: number) => {
    completeGoal.mutate({ goalId, satisfaction })
  }

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <div className="flex items-center gap-2">
          <StarIcon className="h-5 w-5 text-yellow-500" />
          <CardTitle className="text-base">Today&apos;s Big 3</CardTitle>
        </div>
        {goals.length > 0 && (
          <Badge
            variant={completedCount === goals.length ? 'default' : 'outline'}
            className={completedCount === goals.length ? 'bg-green-500' : ''}
          >
            {completedCount}/{goals.length}
          </Badge>
        )}
      </CardHeader>
      <CardContent className="space-y-3">
        {/* Progress bar */}
        {goals.length > 0 && (
          <div className="h-2 bg-muted rounded-full overflow-hidden">
            <div
              className="h-full bg-gradient-to-r from-yellow-500 to-green-500 transition-all duration-500"
              style={{ width: `${progress}%` }}
            />
          </div>
        )}

        {isLoading ? (
          <div className="py-8 text-center">
            <StarIcon className="h-8 w-8 mx-auto text-muted-foreground animate-pulse" />
            <p className="text-sm text-muted-foreground mt-2">
              Loading goals...
            </p>
          </div>
        ) : goals.length === 0 ? (
          <div className="py-4 text-center space-y-3">
            <StarIcon className="h-8 w-8 mx-auto text-yellow-500/50" />
            <p className="text-sm text-muted-foreground">
              Set your 3 most important goals for today
            </p>
            <AddGoalDialog onAdd={handleAddGoal} existingCount={0} />
          </div>
        ) : (
          <>
            {goals.map((goal) => (
              <GoalItem
                key={goal.id}
                goal={goal}
                onComplete={(satisfaction) =>
                  handleComplete(goal.id, satisfaction)
                }
                isLoading={completeGoal.isPending}
              />
            ))}
            <AddGoalDialog onAdd={handleAddGoal} existingCount={goals.length} />
          </>
        )}

        {/* Celebration */}
        {goals.length > 0 && completedCount === goals.length && (
          <div className="text-center py-2">
            <p className="text-sm font-medium text-green-600">
              🎉 Amazing! You completed all your Big 3 today!
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
