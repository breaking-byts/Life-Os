import {
  BrainIcon,
  ChevronDownIcon,
  ChevronRightIcon,
  InfoIcon,
  RefreshCwIcon,
  SparklesIcon,
  ThumbsDownIcon,
  ThumbsUpIcon,
  ZapIcon,
} from 'lucide-react'
import { useState } from 'react'

import { useIntelligence } from '@/hooks/useIntelligence'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import type { AgentRecommendation } from '@/types'

function getConfidenceColor(level: string) {
  switch (level) {
    case 'high':
      return 'bg-green-500/20 text-green-600 dark:text-green-400'
    case 'medium':
      return 'bg-yellow-500/20 text-yellow-600 dark:text-yellow-400'
    case 'low':
      return 'bg-muted text-muted-foreground'
    default:
      return 'bg-muted'
  }
}

function getCategoryIcon(category: string) {
  switch (category) {
    case 'productivity':
      return <ZapIcon className="h-4 w-4" />
    case 'physical':
      return <span>💪</span>
    case 'wellness':
      return <span>🧘</span>
    case 'learning':
      return <span>📚</span>
    default:
      return <SparklesIcon className="h-4 w-4" />
  }
}

function RecommendationCard({
  recommendation,
  isPrimary = false,
  onAccept,
  onFeedback,
  isLoading,
}: {
  recommendation: AgentRecommendation
  isPrimary?: boolean
  onAccept: () => void
  onFeedback: (positive: boolean) => void
  isLoading: boolean
}) {
  const [showDetails, setShowDetails] = useState(false)
  const [showAlternatives, setShowAlternatives] = useState(false)

  return (
    <div
      className={`rounded-lg border p-4 transition-all ${
        isPrimary
          ? 'border-primary/50 bg-primary/5'
          : 'border-border bg-muted/40 hover:bg-muted/60'
      }`}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-start gap-3 flex-1">
          <div className="mt-0.5">
            {getCategoryIcon(recommendation.action.category)}
          </div>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <h4 className="font-medium">
                {recommendation.action.name.replace(/_/g, ' ')}
              </h4>
              <Badge variant="outline" className="text-xs capitalize">
                {recommendation.action.category}
              </Badge>
              <Badge
                className={`text-xs ${getConfidenceColor(recommendation.confidence_level)}`}
              >
                {recommendation.confidence_level} confidence
              </Badge>
            </div>
            <p className="text-sm text-muted-foreground mt-1">
              {recommendation.explanation}
            </p>

            {/* Feature contributions */}
            {showDetails && recommendation.top_features.length > 0 && (
              <div className="mt-3 space-y-2">
                <p className="text-xs font-medium text-muted-foreground">
                  Why this recommendation:
                </p>
                <div className="space-y-1">
                  {recommendation.top_features.slice(0, 3).map((feature) => (
                    <div
                      key={feature.name}
                      className="flex items-center gap-2 text-xs"
                    >
                      <span
                        className={`w-2 h-2 rounded-full ${
                          feature.direction === 'positive'
                            ? 'bg-green-500'
                            : 'bg-red-500'
                        }`}
                      />
                      <span className="text-muted-foreground">
                        {feature.name.replace(/_/g, ' ')}
                      </span>
                      <Progress
                        value={Math.abs(feature.contribution) * 100}
                        className="h-1.5 w-16"
                      />
                    </div>
                  ))}
                </div>

                {/* Similar experiences */}
                {recommendation.similar_experiences.length > 0 && (
                  <div className="mt-2 pt-2 border-t border-border/50">
                    <p className="text-xs font-medium text-muted-foreground mb-1">
                      Similar past experiences:
                    </p>
                    {recommendation.similar_experiences
                      .slice(0, 2)
                      .map((exp, i) => (
                        <p key={i} className="text-xs text-muted-foreground">
                          • {exp.description.slice(0, 60)}...
                          <span className="ml-1 text-green-600">
                            ({Math.round(exp.outcome * 100)}% success)
                          </span>
                        </p>
                      ))}
                  </div>
                )}

                {/* Stats */}
                <div className="flex gap-4 mt-2 pt-2 border-t border-border/50 text-xs text-muted-foreground">
                  <span>
                    Expected reward:{' '}
                    {Math.round(recommendation.expected_reward * 100)}%
                  </span>
                  <span>
                    Uncertainty: ±{Math.round(recommendation.uncertainty * 100)}
                    %
                  </span>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Actions */}
        <div className="flex items-center gap-1 shrink-0">
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7"
                onClick={() => setShowDetails(!showDetails)}
              >
                {showDetails ? (
                  <ChevronDownIcon className="h-4 w-4" />
                ) : (
                  <InfoIcon className="h-4 w-4" />
                )}
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
              <p className="text-xs">
                {showDetails ? 'Hide details' : 'Show why'}
              </p>
            </TooltipContent>
          </Tooltip>

          {isPrimary && (
            <Button
              variant="default"
              size="sm"
              className="h-7"
              onClick={onAccept}
              disabled={isLoading}
            >
              Do it
            </Button>
          )}

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7"
                onClick={() => onFeedback(true)}
                disabled={isLoading}
              >
                <ThumbsUpIcon className="h-3.5 w-3.5" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
              <p className="text-xs">Good suggestion</p>
            </TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7"
                onClick={() => onFeedback(false)}
                disabled={isLoading}
              >
                <ThumbsDownIcon className="h-3.5 w-3.5" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
              <p className="text-xs">Not helpful</p>
            </TooltipContent>
          </Tooltip>
        </div>
      </div>

      {/* Alternatives */}
      {isPrimary && recommendation.alternatives.length > 0 && (
        <div className="mt-3">
          <button
            onClick={() => setShowAlternatives(!showAlternatives)}
            className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
          >
            <ChevronRightIcon
              className={`h-3 w-3 transition-transform ${showAlternatives ? 'rotate-90' : ''}`}
            />
            {recommendation.alternatives.length} alternatives
          </button>
          {showAlternatives && (
            <div className="pt-2 space-y-1">
              {recommendation.alternatives.map((alt, i) => (
                <div
                  key={i}
                  className="flex items-center justify-between p-2 rounded bg-muted/40 text-sm"
                >
                  <span>{alt.action.name.replace(/_/g, ' ')}</span>
                  <span className="text-xs text-muted-foreground">
                    {Math.round(alt.expected_reward * 100)}% expected
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}

export function AgentRecommendations() {
  const {
    recommendations,
    isLoadingRecommendations,
    refetchRecommendations,
    isRefetchingRecommendations,
    recordFeedback,
    status,
  } = useIntelligence()

  const handleAccept = (recommendation: AgentRecommendation) => {
    if (recommendation.recommendation_id) {
      recordFeedback.mutate({
        recommendationId: recommendation.recommendation_id,
        accepted: true,
        outcomeScore: 0.8, // Default positive outcome
      })
    }
  }

  const handleFeedback = (
    recommendation: AgentRecommendation,
    positive: boolean,
  ) => {
    if (recommendation.recommendation_id) {
      recordFeedback.mutate({
        recommendationId: recommendation.recommendation_id,
        accepted: false,
        feedbackScore: positive ? 1 : -1,
      })
    }
  }

  return (
    <TooltipProvider>
      <Card className="border-primary/30">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <div className="flex items-center gap-2">
            <BrainIcon className="h-5 w-5 text-primary" />
            <CardTitle className="text-base">AI Recommendations</CardTitle>
            {status && (
              <Tooltip>
                <TooltipTrigger>
                  <Badge variant="outline" className="text-xs">
                    {status.mode}
                  </Badge>
                </TooltipTrigger>
                <TooltipContent>
                  <div className="text-xs space-y-1">
                    <p>Mode: {status.mode}</p>
                    <p>Samples: {status.total_samples}</p>
                    <p>Accuracy: {Math.round(status.avg_accuracy * 100)}%</p>
                  </div>
                </TooltipContent>
              </Tooltip>
            )}
          </div>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={() => refetchRecommendations()}
            disabled={isRefetchingRecommendations}
          >
            <RefreshCwIcon
              className={`h-4 w-4 ${isRefetchingRecommendations ? 'animate-spin' : ''}`}
            />
          </Button>
        </CardHeader>
        <CardContent className="space-y-3">
          {isLoadingRecommendations ? (
            <div className="py-8 text-center">
              <BrainIcon className="h-8 w-8 mx-auto text-muted-foreground animate-pulse" />
              <p className="text-sm text-muted-foreground mt-2">
                Analyzing your context...
              </p>
            </div>
          ) : recommendations.length === 0 ? (
            <div className="py-8 text-center">
              <SparklesIcon className="h-8 w-8 mx-auto text-muted-foreground" />
              <p className="text-sm text-muted-foreground mt-2">
                No recommendations yet. Keep using the app to train the agent!
              </p>
            </div>
          ) : (
            recommendations.map((rec, i) => (
              <RecommendationCard
                key={rec.recommendation_id ?? i}
                recommendation={rec}
                isPrimary={i === 0}
                onAccept={() => handleAccept(rec)}
                onFeedback={(positive) => handleFeedback(rec, positive)}
                isLoading={recordFeedback.isPending}
              />
            ))
          )}
        </CardContent>
      </Card>
    </TooltipProvider>
  )
}
