# Kroma Frontend

React + TypeScript frontend for Kroma comic/graphic-novel production tool.

## Tech Stack

- **React 19** with TypeScript
- **Vite** for build tooling and dev server
- **React Bootstrap** for UI components
- **Material Icons** for iconography
- **React Router** for navigation

## Journey Implementation

This frontend implements the Kroma user journey from `docs/USER_FLOW_JOURNEY_MAP.md`:

### Completed (J00-J01)

- **J00** - Onboarding and Provider Setup
  - `/` - Onboarding page with API token bootstrap
- **J01** - Create or Select Project Universe
  - `/projects` - Project list and creation
  - `/projects/:slug` - Project detail view

### Planned

- **J02** - Build Continuity References (style guides, characters, reference sets)
- **J03** - Bootstrap Story Settings
- **J04-J07** - Run workflows (style lock, time/weather, character identity, post-process)
- **J08** - Review, Curate, and Export

## Development

```bash
# Install dependencies
npm install

# Start dev server (backend must be running on 127.0.0.1:8788)
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## Environment Variables

Create `.env` file:

```env
VITE_API_BASE_URL=http://127.0.0.1:8788
```

## API Integration

The frontend communicates with the Rust backend via REST API:
- Health check: `GET /health`
- Auth bootstrap: `POST /api/auth/token`
- Projects: `GET/POST /api/projects`
- Project detail: `GET /api/projects/{slug}`

See `src/api/client.ts` for the API client implementation.

## Project Structure

```
src/
├── api/
│   └── client.ts       # API client with TypeScript types
├── components/
│   └── Layout.tsx      # Main app layout with navbar
├── pages/
│   ├── OnboardingPage.tsx    # J00: Auth bootstrap
│   ├── ProjectsPage.tsx      # J01: Project list/create
│   └── ProjectDetailPage.tsx # J01: Project detail
├── App.tsx           # Main app component with routing
└── main.tsx          # Entry point
```
