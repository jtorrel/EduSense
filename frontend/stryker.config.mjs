// @ts-check
/**
 * Configuration Stryker (ADR 0012) : tout mutant survivant fait échouer le contrôle.
 * @type {import('@stryker-mutator/api/core').PartialStrykerOptions}
 */
const config = {
  testRunner: 'vitest',
  plugins: ['@stryker-mutator/vitest-runner'],
  // Mêmes exclusions que la couverture (ADR 0011).
  mutate: ['src/**/*.{ts,tsx}', '!src/main.tsx', '!src/**/*.test.{ts,tsx}'],
  reporters: ['clear-text', 'progress', 'html'],
  htmlReporter: { fileName: 'reports/mutation/index.html' },
  thresholds: { high: 100, low: 100, break: 100 },
  incremental: false,
}

export default config
