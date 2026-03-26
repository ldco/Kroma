/**
 * GET /api/docs
 *
 * Serves Swagger UI for interactive API documentation.
 * Uses CDN-hosted Swagger UI assets.
 * Public endpoint - no authentication required.
 */
export default defineEventHandler((event) => {
  // Generate a simple nonce using timestamp + random
  const nonce = Buffer.from(Date.now().toString() + Math.random().toString()).toString('base64').slice(0, 24)

  const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Puppet Master API Documentation</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css">
  <style nonce="${nonce}">
    body { margin: 0; padding: 0; }
    .swagger-ui .topbar { display: none; }
    .swagger-ui .info { margin: 20px 0; }
    .swagger-ui .info .title { font-size: 2rem; }
  </style>
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script nonce="${nonce}">
    window.onload = function() {
      SwaggerUIBundle({
        url: '/api/docs/openapi.json',
        dom_id: '#swagger-ui',
        presets: [SwaggerUIBundle.presets.apis, SwaggerUIBundle.SwaggerUIStandalonePreset],
        layout: 'BaseLayout',
        deepLinking: true,
        docExpansion: 'list',
        filter: true,
        tryItOutEnabled: true
      });
    };
  </script>
</body>
</html>`

  // Set CSP header with nonce
  const csp = [
    "default-src 'self'",
    `script-src 'self' 'nonce-${nonce}' https://cdn.jsdelivr.net`,
    `style-src 'self' 'nonce-${nonce}' https://cdn.jsdelivr.net`,
    "img-src 'self' data: https:",
    "connect-src 'self'",
    "frame-ancestors 'none'"
  ].join('; ')

  setHeader(event, 'Content-Type', 'text/html')
  setHeader(event, 'Content-Security-Policy', csp)
  return html
})
