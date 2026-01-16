import { useQuery } from '@tanstack/react-query'
import { WeeklyReviewForm } from './weekly-review-form'
import { tauri } from '@/lib/tauri'
import { formatDate } from '@/lib/time'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

export function WeeklyReviewList() {
  const weeklyReviewsQuery = useQuery({
    queryKey: ['weekly-reviews'],
    queryFn: tauri.getWeeklyReviews,
  })

  const reviews = weeklyReviewsQuery.data ?? []

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Weekly Reviews</CardTitle>
        <WeeklyReviewForm />
      </CardHeader>
      <CardContent className="space-y-3">
        {reviews.length === 0 && (
          <p className="text-muted-foreground text-sm">No reviews yet.</p>
        )}
        {reviews.map((review) => (
          <div
            key={review.id}
            className="border-border bg-muted/60 rounded-md border p-3"
          >
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">Week of {formatDate(review.week_start)}</p>
                {review.wins && (
                  <p className="text-muted-foreground text-sm">Wins: {review.wins}</p>
                )}
              </div>
              <p className="text-muted-foreground text-xs">
                Created {formatDate(review.created_at, 'MMM d')}
              </p>
            </div>
            {review.improvements && (
              <p className="text-muted-foreground text-sm">Focus: {review.improvements}</p>
            )}
          </div>
        ))}
      </CardContent>
    </Card>
  )
}
