/**
 * GET /api/docs/openapi.json
 *
 * Serves the OpenAPI 3.0 specification as JSON.
 * Public endpoint - no authentication required.
 */
import { openApiSpec } from '../../openapi'

export default defineEventHandler(() => {
  return openApiSpec
})
