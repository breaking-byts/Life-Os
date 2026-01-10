import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { createRouter } from '@tanstack/react-router'
import { useState } from 'react'

// Import the generated route tree
import { routeTree } from './routeTree.gen'

const queryClient = new QueryClient()

// Create a new router instance
export const getRouter = () => {
  const router = createRouter({
    routeTree,
    scrollRestoration: true,
    defaultPreloadStaleTime: 0,
    Wrap: ({ children }) => {
      const [client] = useState(() => queryClient)
      return <QueryClientProvider client={client}>{children}</QueryClientProvider>
    },
  })

  return router
}
