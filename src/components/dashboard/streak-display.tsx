import { useQuery } from '@tanstack/react-query'
import { BookOpenIcon, DumbbellIcon, FlameIcon, SmileIcon, TargetIcon } from 'lucide-react'
import { tauri } from '@/lib/tauri'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

const STREAK_KEY = ['streaks']

export function StreakDisplay() {
    const { data: streaks, isLoading } = useQuery({
        queryKey: STREAK_KEY,
        queryFn: tauri.getStreaks,
    })

    if (isLoading) {
        return (
            <Card>
                <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                        <FlameIcon className="h-5 w-5 text-orange-500" />
                        Streaks
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <p className="text-muted-foreground text-sm">Loading...</p>
                </CardContent>
            </Card>
        )
    }

    const items = [
        { label: 'Study', value: streaks?.study_streak ?? 0, icon: BookOpenIcon, color: 'text-blue-500' },
        { label: 'Workout', value: streaks?.workout_streak ?? 0, icon: DumbbellIcon, color: 'text-green-500' },
        { label: 'Practice', value: streaks?.practice_streak ?? 0, icon: TargetIcon, color: 'text-purple-500' },
        { label: 'Check-in', value: streaks?.checkin_streak ?? 0, icon: SmileIcon, color: 'text-amber-500' },
    ]

    const maxStreak = Math.max(...items.map(i => i.value))

    return (
        <Card>
            <CardHeader>
                <CardTitle className="flex items-center gap-2">
                    <FlameIcon className="h-5 w-5 text-orange-500" />
                    Streaks
                </CardTitle>
            </CardHeader>
            <CardContent>
                <div className="grid grid-cols-2 gap-4">
                    {items.map((item) => (
                        <div
                            key={item.label}
                            className={`flex items-center gap-3 rounded-lg border p-3 ${item.value === maxStreak && item.value > 0
                                    ? 'border-orange-500/50 bg-orange-500/5'
                                    : 'border-border'
                                }`}
                        >
                            <item.icon className={`h-5 w-5 ${item.color}`} />
                            <div>
                                <p className="text-2xl font-bold">
                                    {item.value}
                                    {item.value > 0 && <span className="ml-1 text-sm">🔥</span>}
                                </p>
                                <p className="text-muted-foreground text-xs">{item.label}</p>
                            </div>
                        </div>
                    ))}
                </div>
                {maxStreak > 0 && (
                    <p className="text-muted-foreground mt-4 text-center text-sm">
                        Keep it up! Consistency is key 💪
                    </p>
                )}
            </CardContent>
        </Card>
    )
}
