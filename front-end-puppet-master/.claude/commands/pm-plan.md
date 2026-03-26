# /pm-plan — Create Development Plan from Brief

Analyzes the project technical brief and creates a structured development plan.

## When to Use

- After running `/pm-init` and completing the setup wizard
- When you have a technical brief in `.claude-data/project-brief.md`
- To create a roadmap before starting development

## Algorithm

### 1. Load Project Context

Read the following files to understand the project:

```
Read .claude-data/project-brief.md     # Technical brief from wizard
Read app/puppet-master.config.ts       # Project configuration
Read .claude-data/context.md           # Session context (if exists)
```

### 2. Analyze Technical Brief

From the brief, extract:
- **Project goals**: What is being built
- **Features requested**: Specific functionality needed
- **Custom modules**: Any modules described in the brief
- **Technical requirements**: Stack preferences, integrations, constraints
- **Target audience**: Who will use this

### 3. Generate Development Plan

Create a phased plan based on PM best practices:

**Phase 1: Foundation**
- Database schema design (if custom entities needed)
- Core component structure
- API route scaffolding

**Phase 2: Core Features**
- Implement primary features from brief
- Build custom modules
- Connect frontend to API

**Phase 3: Polish**
- Styling and UX improvements
- Error handling
- Testing

**Phase 4: Launch Prep**
- SEO optimization (if website)
- Performance audit
- Security review

### 4. Create Todo List

Use TodoWrite to create actionable tasks from the plan:

```typescript
TodoWrite([
  { content: "Task from plan", status: "pending", activeForm: "Working on task" },
  // ... more tasks
])
```

### 5. Present Plan to User

Output the plan in this format:

```
## Development Plan: {Project Name}

Based on your technical brief, here's the recommended approach:

### Phase 1: Foundation
- [ ] Task 1
- [ ] Task 2

### Phase 2: Core Features
- [ ] Task 3
- [ ] Task 4

### Phase 3: Polish
- [ ] Task 5

---

**Estimated scope**: {X} major tasks across {Y} phases

Ready to start? Pick a task or say "start" to begin with the first one.
```

### 6. Handle User Response

- If user says "start" → Begin first task
- If user picks specific task → Begin that task
- If user asks questions → Clarify and adjust plan

## Example Output

```
## Development Plan: My Awesome SaaS

Based on your technical brief, here's the recommended approach:

### Phase 1: Foundation (Database & API)
- [ ] Create user profiles schema with subscription tiers
- [ ] Build authentication API with JWT tokens
- [ ] Set up Stripe integration for payments

### Phase 2: Core Features (Dashboard)
- [ ] Build dashboard layout with sidebar navigation
- [ ] Implement usage metrics cards
- [ ] Create data visualization charts
- [ ] Add export to CSV/PDF functionality

### Phase 3: Polish (UX & Performance)
- [ ] Add loading states and skeleton screens
- [ ] Implement error boundaries
- [ ] Add toast notifications

---

**Estimated scope**: 10 tasks across 3 phases

Ready to start? Pick a task or say "start" to begin with Phase 1.
```

## No Brief Found

If `.claude-data/project-brief.md` doesn't exist:

```
No technical brief found.

Options:
1. Run `/pm-init` to configure project and add a brief
2. Tell me what you want to build and I'll create a plan
```

## Notes

- The plan adapts based on project type (website vs app)
- For websites: prioritizes content, SEO, modules
- For apps: prioritizes auth, dashboard, user features
- Always respects modules selected in wizard
- Considers enabled features (PWA, i18n, dark mode, etc.)
