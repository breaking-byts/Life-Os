import { CheckinForm } from './checkin-form'
import { useCheckIn } from '@/hooks/useCheckIn'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Separator } from '@/components/ui/separator'
import { formatDate } from '@/lib/time'

const todayTasks = [
  { title: 'Deep work block', detail: '90 min on priority course' },
  { title: 'Skill reps', detail: '45 min deliberate practice' },
  { title: 'Recovery', detail: 'Evening walk + 10m mobility' },
]

export function TodayView() {
  const today = formatDate(new Date(), 'EEEE, MMM d')
  const { checkInQuery } = useCheckIn()
  const checkIn = checkInQuery.data
  const mood = checkIn?.mood
  const energy = checkIn?.energy

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>Today</CardTitle>
          <div className="flex items-center gap-2">
            {!checkIn && <CheckinForm />}
            <Badge variant="secondary">{today}</Badge>
          </div>
        </div>
        {(mood || energy) && (
          <div className="text-muted-foreground flex gap-3 text-sm">
            {mood && <span>Mood: {mood}/10</span>}
            {energy && <span>Energy: {energy}/10</span>}
          </div>
        )}
      </CardHeader>
      <CardContent className="space-y-4">
        {todayTasks.map((item, index) => (
          <div key={item.title} className="space-y-1">
            <div className="flex items-center gap-2 text-sm font-medium">
              <span className="text-muted-foreground">{index + 1}.</span>
              {item.title}
            </div>
            <p className="text-muted-foreground text-sm">{item.detail}</p>
            {index < todayTasks.length - 1 && <Separator className="pt-2" />}
          </div>
        ))}
      </CardContent>
    </Card>
  )
}
