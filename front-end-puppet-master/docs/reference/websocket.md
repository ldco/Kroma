# WebSocket Integration Guide

Real-time communication foundation for PuppetMaster2.

## Overview

The WebSocket system provides:
- Authenticated connections using existing session cookies
- Room-based messaging with role-based access control
- Rate limiting and connection limits
- Auto-reconnect with exponential backoff
- Type-safe message protocol

## Architecture

```
Client (Browser)                    Server (Nitro/H3)
─────────────────                   ─────────────────
useWebSocket()  ──── ws:// ────►   /api/ws
composable                              │
    │                                   ├── Auth (pm-session cookie)
    │                                   ├── Room management
    │                                   └── Message routing
    │                                        │
    ▼                                        ▼
Vue Component  ◄──── events ────────  WebSocket Hub (in-memory)
```

## Quick Start

### Client-Side Usage

```typescript
// In any Vue component
const { connected, subscribe, sendToRoom, on, connect } = useWebSocket()

// Connect (or use autoConnect option)
connect()

// Watch connection state
watch(connected, (isConnected) => {
  if (isConnected) {
    console.log('WebSocket connected!')
  }
})

// Subscribe to a room
subscribe('support')

// Listen for messages
const unsubscribe = on('message', (payload) => {
  console.log('Received:', payload)
})

// Send message to room
sendToRoom('support', { text: 'Hello!' })

// Clean up listener when done
onUnmounted(() => {
  unsubscribe()
})
```

### Auto-Connect Option

```typescript
// Auto-connect when component mounts
const { connected, subscribe } = useWebSocket({
  autoConnect: true,
  reconnect: true,           // Auto-reconnect on disconnect (default: true)
  maxReconnectAttempts: 5,   // Max reconnection attempts (default: 5)
  reconnectDelay: 1000       // Base delay in ms (default: 1000)
})
```

## Message Types

### Built-in Message Types

| Type | Direction | Description |
|------|-----------|-------------|
| `ping` | Client → Server | Heartbeat request |
| `pong` | Server → Client | Heartbeat response |
| `connected` | Server → Client | Connection confirmed with user info |
| `error` | Server → Client | Error notification |
| `subscribe` | Client → Server | Join a room |
| `unsubscribe` | Client → Server | Leave a room |
| `subscribed` | Server → Client | Room join confirmed |
| `unsubscribed` | Server → Client | Room leave confirmed |
| `message` | Bidirectional | Generic message to/from room |
| `notification` | Server → Client | Push notification |

### Message Structure

```typescript
interface WSMessage<T = unknown> {
  type: string        // Message type identifier
  payload?: T         // Type-specific data
  timestamp: number   // Unix timestamp (ms)
  id?: string         // Optional ID for acknowledgment
}
```

## Rooms

Rooms isolate message groups. Users must subscribe before sending/receiving.

### Predefined Rooms

| Room | Min Role | Description |
|------|----------|-------------|
| `support` | editor | Support chat channel |
| `admin:notifications` | admin | Admin notifications |
| `system:broadcast` | master | System-wide broadcasts |

### Custom Rooms

Any room name is allowed. Rooms without predefined config are open to all authenticated users.

```typescript
// Subscribe to custom room
subscribe('project:123')

// Send to custom room
sendToRoom('project:123', { action: 'update', data: {...} })
```

### Adding Room Restrictions

Edit `server/utils/websocket/types.ts`:

```typescript
export const ROOM_CONFIG: Record<string, WSRoomOptions> = {
  // Add your room
  'my-room': {
    minRole: 'admin',      // Minimum role required
    maxConnections: 100    // Optional capacity limit
  }
}
```

## Server-Side Usage

### Broadcasting from API Routes

```typescript
// In any API handler
import { wsHub } from '../utils/websocket'
import { createWSMessage, WS_MESSAGE_TYPES } from '../../types/websocket'

export default defineEventHandler(async (event) => {
  // ... handle request ...

  // Broadcast to room
  wsHub.broadcastToRoom('support', createWSMessage(
    WS_MESSAGE_TYPES.MESSAGE,
    { type: 'new_ticket', ticket: {...} }
  ))

  // Send to specific user
  wsHub.sendToUser(userId, createWSMessage(
    WS_MESSAGE_TYPES.NOTIFICATION,
    { title: 'New message', body: 'You have a new support message' }
  ))

  // Broadcast to all admins
  wsHub.broadcastToAuthenticated(
    createWSMessage(WS_MESSAGE_TYPES.NOTIFICATION, { title: 'Alert' }),
    'admin' // minimum role
  )

  return { success: true }
})
```

### Hub API Reference

```typescript
import { wsHub } from '../utils/websocket'

// Connection management
wsHub.addConnection(peer, user)           // Add new connection
wsHub.removeConnection(connectionId)      // Remove connection
wsHub.getConnection(connectionId)         // Get connection by ID
wsHub.getUserConnections(userId)          // Get all connections for user

// Room management
wsHub.subscribe(connectionId, room)       // Subscribe to room
wsHub.unsubscribe(connectionId, room)     // Unsubscribe from room

// Messaging
wsHub.send(connectionId, message)         // Send to specific connection
wsHub.sendToUser(userId, message)         // Send to all user connections
wsHub.broadcastToRoom(room, message)      // Broadcast to room
wsHub.broadcastToAuthenticated(message)   // Broadcast to all authenticated

// Utilities
wsHub.getStats()                          // Get hub statistics
wsHub.checkRateLimit(connectionId, max, window)  // Check rate limit
```

## Client Composable API

```typescript
const {
  // Reactive State
  connected,      // Ref<boolean> - Connection status
  connectionId,   // Ref<string | null> - Server-assigned connection ID
  userId,         // Ref<number | null> - Authenticated user ID

  // Connection
  connect(),      // Establish connection
  disconnect(),   // Close connection

  // Messaging
  send(message),              // Send raw WSMessage
  sendMessage(type, payload), // Send typed message
  sendToRoom(room, content),  // Send to room (convenience)
  ping(),                     // Send heartbeat

  // Rooms
  subscribe(room),    // Join room
  unsubscribe(room),  // Leave room

  // Events
  on(type, handler),   // Register handler, returns unsubscribe fn
  once(type, handler)  // One-time handler
} = useWebSocket(options)
```

## Security

### Authentication

WebSocket connections authenticate using the existing `pm-session` cookie:
- Cookie is automatically sent during WebSocket upgrade
- Server validates session and attaches user context
- Unauthenticated connections allowed (for public rooms)

### Rate Limiting

Default limits (configurable in `server/utils/websocket/types.ts`):
- 60 messages per minute per connection
- 5 connections per user
- 10 room subscriptions per connection

### CSRF

WebSocket connections are exempt from CSRF (uses cookie-based auth on upgrade).
Message-level CSRF is not required as connections are already authenticated.

## Limits & Configuration

Edit `server/utils/websocket/types.ts`:

```typescript
// Rate limiting
export const WS_RATE_LIMIT: WSRateLimitConfig = {
  maxMessages: 60,      // Messages per window
  windowMs: 60_000      // Window duration (1 minute)
}

// Connection limits
export const WS_LIMITS = {
  maxConnectionsPerUser: 5,
  maxRoomsPerConnection: 10,
  heartbeatIntervalMs: 30_000,
  heartbeatTimeoutMs: 10_000
}
```

## Example: Support Chat

### Admin Panel Component

```vue
<script setup lang="ts">
const { connected, subscribe, sendToRoom, on } = useWebSocket({ autoConnect: true })

const messages = ref<Array<{ from: string; text: string; time: Date }>>([])
const newMessage = ref('')

// Subscribe to support room when connected
watch(connected, (isConnected) => {
  if (isConnected) {
    subscribe('support')
  }
})

// Listen for incoming messages
on('message', (payload: any) => {
  if (payload.room === 'support') {
    messages.value.push({
      from: payload.from?.name || 'Anonymous',
      text: payload.content.text,
      time: new Date()
    })
  }
})

function send() {
  if (!newMessage.value.trim()) return

  sendToRoom('support', { text: newMessage.value })
  newMessage.value = ''
}
</script>

<template>
  <div class="support-chat">
    <div class="status">
      {{ connected ? 'Connected' : 'Disconnected' }}
    </div>

    <div class="messages">
      <div v-for="msg in messages" :key="msg.time.getTime()" class="message">
        <strong>{{ msg.from }}:</strong> {{ msg.text }}
      </div>
    </div>

    <form @submit.prevent="send">
      <input v-model="newMessage" placeholder="Type message..." />
      <button type="submit">Send</button>
    </form>
  </div>
</template>
```

### Server-Side: Notify on New Contact Submission

```typescript
// In server/api/contact/submit.post.ts
import { wsHub } from '../../utils/websocket'
import { createWSMessage, WS_MESSAGE_TYPES } from '../../../types/websocket'

export default defineEventHandler(async (event) => {
  // ... save contact submission ...

  // Notify admins via WebSocket
  wsHub.broadcastToRoom('support', createWSMessage(
    WS_MESSAGE_TYPES.NOTIFICATION,
    {
      title: 'New Contact Submission',
      body: `From: ${submission.name}`,
      type: 'info'
    }
  ))

  return { success: true }
})
```

## File Structure

```
app/
├── types/
│   └── websocket.ts              # Shared message types
├── server/
│   ├── api/
│   │   └── ws.ts                 # WebSocket endpoint
│   └── utils/
│       └── websocket/
│           ├── index.ts          # Exports
│           ├── types.ts          # Server types & config
│           ├── hub.ts            # Connection hub
│           └── auth.ts           # WS authentication
└── app/
    └── composables/
        └── useWebSocket.ts       # Client composable
```

## Troubleshooting

### Connection Issues

1. **426 Upgrade Required**: Normal for HTTP requests to `/api/ws`. WebSocket clients handle upgrade automatically.

2. **Connection drops immediately**: Check browser console for errors. Verify session cookie is being sent.

3. **Rate limited**: Reduce message frequency or adjust `WS_RATE_LIMIT` config.

### Authentication Issues

1. **Can't join room**: Check user role meets room's `minRole` requirement.

2. **Not authenticated**: Ensure user is logged in (has valid `pm-session` cookie).

### Debugging

Enable debug logging in development:

```typescript
// Client-side: messages are logged to console
on('*', (payload, message) => {
  console.log('[WS]', message.type, payload)
})
```

Server logs WebSocket events at DEBUG level. Set `LOG_LEVEL=debug` to see them.
