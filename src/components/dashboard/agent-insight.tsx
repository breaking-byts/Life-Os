import { useQuery } from '@tanstack/react-query'
import { BrainIcon, RefreshCwIcon } from 'lucide-react'
import { tauri } from '@/lib/tauri'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'

export function AgentInsight() {
  const { data: insights = [], isLoading, refetch, isRefetching } = useQuery({
    queryKey: ['insights'],
    queryFn: tauri.getInsights,
    staleTime: 5 * 60 * 1000, // 5 minutes
  })

  return (
    <Card className="border-primary/30">
      <CardHeader className="flex flex-row items-center justify-between space-y-0">
        <div className="flex items-center gap-2">
          <BrainIcon className="h-5 w-5 text-primary" />
          <CardTitle>Agent Insights</CardTitle>
        </div>
        <Button
          variant="ghost"
          size="icon-sm"
          onClick={() => refetch()}
          disabled={isRefetching}
        >
          <RefreshCwIcon className={`h-4 w-4 ${isRefetching ? 'animate-spin' : ''}`} />
        </Button>
      </CardHeader>
      <CardContent className="space-y-2">
        {isLoading ? (
          <p className="text-muted-foreground text-sm">Analyzing your data...</p>
        ) : (
          insights.map((insight, index) => (
            <div
              key={index}
              className="flex items-start gap-3 rounded-md bg-muted/60 p-3 text-sm"
            >
              <span className="text-lg">{insight.icon}</span>
              <div className="flex-1">
                <p>{insight.message}</p>
                <Badge variant="outline" className="mt-1 text-xs">
                  {insight.category}
                </Badge>
              </div>
            </div>
          ))
        )}
      </CardContent>
    </Card>
  )
}
