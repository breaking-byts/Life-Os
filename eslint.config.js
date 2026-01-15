//  @ts-check

import { tanstackConfig } from '@tanstack/eslint-config'

export default [
  ...tanstackConfig,
  {
    ignores: ['eslint.config.js', 'prettier.config.js'],
  },
  {
    rules: {
      // Allow defensive null checks for API responses where runtime values may be null
      // even if TypeScript types say otherwise
      '@typescript-eslint/no-unnecessary-condition': 'off',
    },
  },
]
