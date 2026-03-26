// Lighthouse CI Configuration
// Comment 13: Score thresholds for CI gating

module.exports = {
  ci: {
    collect: {
      // Run 3 times for statistical significance
      numberOfRuns: 3,
      // Test the production URL
      url: ['http://localhost:3000/en'],
      // Use production server
      startServerCommand: 'node .output/server/index.mjs',
      // Give server time to start
      startServerReadyPattern: 'Listening on',
      startServerReadyTimeout: 30000,
    },
    assert: {
      // Use recommended preset as baseline
      preset: 'lighthouse:recommended',
      // Override with project-specific thresholds
      assertions: {
        // Performance score must be >= 80 (recovery target)
        'categories:performance': ['error', { minScore: 0.8 }],
        // Accessibility must be perfect
        'categories:accessibility': ['error', { minScore: 0.95 }],
        // Best practices
        'categories:best-practices': ['error', { minScore: 0.9 }],
        // SEO
        'categories:seo': ['error', { minScore: 0.9 }],
        // Core Web Vitals budgets
        'largest-contentful-paint': ['error', { maxNumericValue: 4000 }],
        'first-contentful-paint': ['error', { maxNumericValue: 2500 }],
        'total-blocking-time': ['error', { maxNumericValue: 300 }],
        'cumulative-layout-shift': ['error', { maxNumericValue: 0.1 }],
        'speed-index': ['warn', { maxNumericValue: 3400 }],
        // Resource budgets
        'resource-summary:document:size': ['warn', { maxNumericValue: 100000 }],
        'resource-summary:script:size': ['warn', { maxNumericValue: 300000 }],
        'resource-summary:stylesheet:size': ['warn', { maxNumericValue: 100000 }],
        'resource-summary:image:size': ['warn', { maxNumericValue: 500000 }],
      },
    },
    upload: {
      // Upload to temporary storage for PR comments
      target: 'temporary-public-storage',
    },
  },
};
