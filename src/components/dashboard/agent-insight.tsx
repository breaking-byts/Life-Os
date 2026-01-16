import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { BrainIcon, RefreshCwIcon, SparklesIcon, ThumbsDownIcon, ThumbsUpIcon } from 'lucide-react'
import { tauri } from '@/lib/tauri'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'

interface Insight {
  icon: string
  message: string
  category: string
  confidence?: number
  insight_id?: number
  arm_name?: string
}

export function AgentInsight() {
  const queryClient = useQueryClient()

  const { data: insights = [], isLoading, refetch, isRefetching } = useQuery({
    queryKey: ['insights'],
    queryFn: tauri.getInsights,
    staleTime: 5 * 60 * 1000, // 5 minutes
  })

  const feedbackMutation = useMutation({
    mutationFn: ({ insightId, actedOn, feedbackScore }: {
      insightId: number
      actedOn: boolean
      feedbackScore?: number
    }) => tauri.recordInsightFeedback(insightId, actedOn, feedbackScore),
    onSuccess: () => {
      // Refetch insights after feedback to potentially get better recommendations
      queryClient.invalidateQueries({ queryKey: ['insights'] })
    }
  })

  const handleFeedback = (insight: Insight, positive: boolean) => {
    if (insight.insight_id) {
      feedbackMutation.mutate({
        insightId: insight.insight_id,
        actedOn: false,
        feedbackScore: positive ? 1 : -1
      })
    }
  }

  const handleActedOn = (insight: Insight) => {
    if (insight.insight_id) {
      feedbackMutation.mutate({
        insightId: insight.insight_id,
        actedOn: true,
        feedbackScore: 1
      })
    }
  }

  const getConfidenceColor = (confidence?: number) => {
    if (!confidence) return 'bg-muted'
    if (confidence >= 0.8) return 'bg-green-500/20 text-green-600 dark:text-green-400'
    if (confidence >= 0.6) return 'bg-yellow-500/20 text-yellow-600 dark:text-yellow-400'
    return 'bg-muted'
  }

  return (
    <TooltipProvider>
      <Card className="border-primary/30">
        <CardHeader className="flex flex-row items-center justify-between space-y-0">
          <div className="flex items-center gap-2">
            <BrainIcon className="h-5 w-5 text-primary" />
            <CardTitle className="text-base">Agent Insights</CardTitle>
            <Tooltip>
              <TooltipTrigger>
                <SparklesIcon className="h-3.5 w-3.5 text-muted-foreground" />
              </TooltipTrigger>
              <TooltipContent>
                <p className="text-xs">Powered by adaptive learning</p>
              </TooltipContent>
            </Tooltip>
          </div>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={() => refetch()}
            disabled={isRefetching}
          >
            <RefreshCwIcon className={`h-4 w-4 ${isRefetching ? 'animate-spin' : ''}`} />
          </Button>
        </CardHeader>
        <CardContent className="space-y-2">
          {isLoading ? (
            <p className="text-muted-foreground text-sm">Analyzing your patterns...</p>
          ) : (
            insights.map((insight, index) => (
              <div
                key={index}
                className="group flex items-start gap-3 rounded-md bg-muted/60 p-3 text-sm transition-colors hover:bg-muted/80"
              >
                <span className="text-lg shrink-0">{insight.icon}</span>
                <div className="flex-1 min-w-0">
                  <p className="text-sm leading-relaxed">{insight.message}</p>
                  <div className="flex items-center gap-2 mt-1.5">
                    <Badge variant="outline" className="text-xs">
                      {insight.category}
                    </Badge>
                    {insight.confidence && (
                      <Badge className={`text-xs ${getConfidenceColor(insight.confidence)}`}>
                        {Math.round(insight.confidence * 100)}% confident
                      </Badge>
                    )}
                  </div>
                </div>
                {insight.insight_id && (
                  <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-6 w-6"
                          onClick={() => handleActedOn(insight)}
                          disabled={feedbackMutation.isPending}
                        >
                          <span className="text-xs">✓</span>
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent side="bottom">
                        <p className="text-xs">I did this!</p>
                      </TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-6 w-6"
                          onClick={() => handleFeedback(insight, true)}
                          disabled={feedbackMutation.isPending}
                        >
                          <ThumbsUpIcon className="h-3 w-3" />
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent side="bottom">
                        <p className="text-xs">Helpful</p>
                      </TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-6 w-6"
                          onClick={() => handleFeedback(insight, false)}
                          disabled={feedbackMutation.isPending}
                        >
                          <ThumbsDownIcon className="h-3 w-3" />
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent side="bottom">
                        <p className="text-xs">Not helpful</p>
                      </TooltipContent>
                    </Tooltip>
                  </div>
                )}
              </div>
            ))
          )}
        </CardContent>
      </Card>
    </TooltipProvider>
  )
}
