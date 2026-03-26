# syntax=docker/dockerfile:1

# ============================================
# PuppetMaster2 - Nuxt 3 Production Dockerfile
# ============================================
# Multi-stage build for optimal image size
# Includes FFmpeg for video processing
# Uses non-root user for security

# ============================================
# Stage 1: Dependencies
# ============================================


FROM node:20-alpine AS deps

WORKDIR /app

# Install build dependencies for native modules
RUN apk add --no-cache \
    libc6-compat \
    python3 \
    make \
    g++

# Copy package files
COPY package.json package-lock.json ./

# Install all dependencies (including devDependencies for build)
RUN npm ci

# ============================================
# Stage 2: Builder
# ============================================
FROM node:20-alpine AS builder

WORKDIR /app

# Copy dependencies from deps stage
COPY --from=deps /app/node_modules ./node_modules

# Copy source code
COPY . .

# Build arguments
ARG NODE_ENV=production
ENV NODE_ENV=${NODE_ENV}

# Build the Nuxt application
RUN npm run build

# ============================================
# Stage 3: Production
# ============================================
FROM node:20-alpine AS production

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache \
    libc6-compat \
    ffmpeg \
    && addgroup --system --gid 1001 nodejs \
    && adduser --system --uid 1001 nuxt

# Copy built application from builder
COPY --from=builder --chown=nuxt:nodejs /app/.output ./.output
COPY --from=builder --chown=nuxt:nodejs /app/package.json ./package.json

# Copy Drizzle config and migrations for runtime migrations
COPY --from=builder --chown=nuxt:nodejs /app/drizzle.config.ts ./drizzle.config.ts
COPY --from=builder --chown=nuxt:nodejs /app/server/database ./server/database

# Create data directory for SQLite
RUN mkdir -p /app/data && chown nuxt:nodejs /app/data

# Create uploads directory
RUN mkdir -p /app/public/uploads && chown nuxt:nodejs /app/public/uploads

# Create temp directory for video processing
RUN mkdir -p /app/data/temp && chown nuxt:nodejs /app/data/temp

# Environment variables
ENV NODE_ENV=production
ENV HOST=0.0.0.0
ENV PORT=3000
ENV DATABASE_URL=/app/data/sqlite.db

# Switch to non-root user
USER nuxt

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/api/health || exit 1

# Start the application
CMD ["node", ".output/server/index.mjs"]
