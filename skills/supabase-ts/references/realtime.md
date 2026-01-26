# Realtime Reference

Supabase Realtime patterns for broadcast, presence, and database changes.

## Table of Contents
- [Channel Types](#channel-types)
- [Broadcast Channels](#broadcast-channels)
- [Presence](#presence)
- [Database Changes](#database-changes)
- [Authorization](#authorization)
- [Connection Management](#connection-management)

## Channel Types

| Type | Use Case | Latency |
|------|----------|---------|
| Broadcast | Chat, cursors, notifications | Lowest |
| Presence | Online users, typing indicators | Low |
| postgres_changes | DB sync, audit logs | Higher |

**Recommendation**: Prefer broadcast for real-time features. Use postgres_changes only when you need DB-triggered events.

## Broadcast Channels

### Basic Broadcast

```typescript
const supabase = getBrowserClient();
if (!supabase) return;

// Create channel
const channel = supabase.channel("room:123");

// Subscribe to events
channel.on("broadcast", { event: "message" }, (payload) => {
  console.log("Message received:", payload);
});

// Subscribe to channel
await channel.subscribe();

// Send message
channel.send({
  type: "broadcast",
  event: "message",
  payload: { text: "Hello!", userId: "user123" },
});
```

### Chat Implementation

```typescript
"use client";

import { useEffect, useState, useCallback } from "react";
import { getBrowserClient } from "@/lib/supabase/client";
import type { RealtimeChannel } from "@supabase/supabase-js";

interface Message {
  id: string;
  text: string;
  userId: string;
  timestamp: number;
}

export function useChat(roomId: string) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [channel, setChannel] = useState<RealtimeChannel | null>(null);

  useEffect(() => {
    const supabase = getBrowserClient();
    if (!supabase) return;

    const ch = supabase.channel(`chat:${roomId}`, {
      config: { private: true },
    });

    ch.on("broadcast", { event: "message" }, ({ payload }) => {
      setMessages((prev) => [...prev, payload as Message]);
    });

    ch.subscribe();
    setChannel(ch);

    return () => {
      ch.unsubscribe();
    };
  }, [roomId]);

  const sendMessage = useCallback(
    (text: string, userId: string) => {
      if (!channel) return;

      const message: Message = {
        id: crypto.randomUUID(),
        text,
        userId,
        timestamp: Date.now(),
      };

      channel.send({
        type: "broadcast",
        event: "message",
        payload: message,
      });
    },
    [channel]
  );

  return { messages, sendMessage };
}
```

### Cursor Sharing

```typescript
"use client";

import { useEffect, useState, useRef } from "react";
import { getBrowserClient } from "@/lib/supabase/client";

interface Cursor {
  x: number;
  y: number;
  userId: string;
}

export function useCursors(roomId: string, userId: string) {
  const [cursors, setCursors] = useState<Map<string, Cursor>>(new Map());
  const channelRef = useRef<RealtimeChannel | null>(null);

  useEffect(() => {
    const supabase = getBrowserClient();
    if (!supabase) return;

    const channel = supabase.channel(`cursors:${roomId}`);

    channel.on("broadcast", { event: "cursor" }, ({ payload }) => {
      setCursors((prev) => {
        const next = new Map(prev);
        next.set(payload.userId, payload);
        return next;
      });
    });

    channel.subscribe();
    channelRef.current = channel;

    return () => {
      channel.unsubscribe();
    };
  }, [roomId]);

  const updateCursor = (x: number, y: number) => {
    channelRef.current?.send({
      type: "broadcast",
      event: "cursor",
      payload: { x, y, userId },
    });
  };

  return { cursors, updateCursor };
}
```

## Presence

### Track Online Users

```typescript
"use client";

import { useEffect, useState } from "react";
import { getBrowserClient } from "@/lib/supabase/client";

interface UserPresence {
  id: string;
  name: string;
  status: "online" | "away";
  lastSeen: number;
}

export function usePresence(roomId: string, currentUser: UserPresence) {
  const [users, setUsers] = useState<UserPresence[]>([]);

  useEffect(() => {
    const supabase = getBrowserClient();
    if (!supabase) return;

    const channel = supabase.channel(`presence:${roomId}`);

    channel
      .on("presence", { event: "sync" }, () => {
        const state = channel.presenceState<UserPresence>();
        const presentUsers = Object.values(state)
          .flat()
          .filter((u) => u.id !== currentUser.id);
        setUsers(presentUsers);
      })
      .on("presence", { event: "join" }, ({ newPresences }) => {
        console.log("User joined:", newPresences);
      })
      .on("presence", { event: "leave" }, ({ leftPresences }) => {
        console.log("User left:", leftPresences);
      })
      .subscribe(async (status) => {
        if (status === "SUBSCRIBED") {
          await channel.track(currentUser);
        }
      });

    return () => {
      channel.unsubscribe();
    };
  }, [roomId, currentUser]);

  return users;
}
```

### Typing Indicator

```typescript
"use client";

import { useEffect, useState, useCallback, useRef } from "react";
import { getBrowserClient } from "@/lib/supabase/client";

export function useTypingIndicator(roomId: string, userId: string) {
  const [typingUsers, setTypingUsers] = useState<string[]>([]);
  const channelRef = useRef<RealtimeChannel | null>(null);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    const supabase = getBrowserClient();
    if (!supabase) return;

    const channel = supabase.channel(`typing:${roomId}`);

    channel.on("broadcast", { event: "typing" }, ({ payload }) => {
      if (payload.userId === userId) return;

      if (payload.isTyping) {
        setTypingUsers((prev) =>
          prev.includes(payload.userId) ? prev : [...prev, payload.userId]
        );
      } else {
        setTypingUsers((prev) =>
          prev.filter((id) => id !== payload.userId)
        );
      }
    });

    channel.subscribe();
    channelRef.current = channel;

    return () => {
      channel.unsubscribe();
    };
  }, [roomId, userId]);

  const setTyping = useCallback(
    (isTyping: boolean) => {
      channelRef.current?.send({
        type: "broadcast",
        event: "typing",
        payload: { userId, isTyping },
      });

      // Auto-clear typing after 3 seconds
      if (isTyping) {
        if (timeoutRef.current) clearTimeout(timeoutRef.current);
        timeoutRef.current = setTimeout(() => {
          setTyping(false);
        }, 3000);
      }
    },
    [userId]
  );

  return { typingUsers, setTyping };
}
```

## Database Changes

### Subscribe to Table Changes

```typescript
const channel = supabase
  .channel("db-changes")
  .on(
    "postgres_changes",
    {
      event: "*",  // INSERT, UPDATE, DELETE
      schema: "public",
      table: "messages",
      filter: `room_id=eq.${roomId}`,
    },
    (payload) => {
      console.log("Change:", payload);
      // payload.eventType: INSERT | UPDATE | DELETE
      // payload.new: new row data
      // payload.old: old row data (UPDATE, DELETE)
    }
  )
  .subscribe();
```

### Event Types

```typescript
// INSERT only
.on("postgres_changes", { event: "INSERT", ... }, handler)

// UPDATE only
.on("postgres_changes", { event: "UPDATE", ... }, handler)

// DELETE only
.on("postgres_changes", { event: "DELETE", ... }, handler)

// All events
.on("postgres_changes", { event: "*", ... }, handler)
```

### Filter Syntax

```typescript
// Equality
filter: "user_id=eq.123"

// Multiple conditions (comma = AND)
filter: "user_id=eq.123,status=eq.active"
```

### Enable Realtime on Table

```sql
-- In migration
alter publication supabase_realtime add table public.messages;

-- Or in dashboard: Database → Publications → supabase_realtime
```

## Authorization

### Private Channels

```typescript
// Client-side
const channel = supabase.channel("private-room", {
  config: {
    private: true,
  },
});
```

### RLS for Realtime

```sql
-- RLS policies apply to postgres_changes
create policy "Users see own messages"
on public.messages for select
to authenticated
using ((select auth.uid()) = user_id);
```

### Broadcast Authorization

For broadcast channels, implement server-side validation:

```typescript
// Server action to validate room access
async function joinRoom(roomId: string) {
  const supabase = await createServerSupabase();
  const { data: { user } } = await supabase.auth.getUser();

  if (!user) throw new Error("Unauthorized");

  // Check room membership
  const { data: membership } = await supabase
    .from("room_members")
    .select("id")
    .eq("room_id", roomId)
    .eq("user_id", user.id)
    .single();

  if (!membership) throw new Error("Not a room member");

  return { canJoin: true };
}
```

## Connection Management

### Connection Status

```typescript
"use client";

import { useEffect, useState } from "react";
import { getBrowserClient } from "@/lib/supabase/client";

type ConnectionStatus = "connecting" | "connected" | "disconnected" | "error";

export function useRealtimeStatus() {
  const [status, setStatus] = useState<ConnectionStatus>("connecting");

  useEffect(() => {
    const supabase = getBrowserClient();
    if (!supabase) return;

    const channel = supabase.channel("connection-status");

    channel.subscribe((status) => {
      switch (status) {
        case "SUBSCRIBED":
          setStatus("connected");
          break;
        case "CLOSED":
          setStatus("disconnected");
          break;
        case "CHANNEL_ERROR":
          setStatus("error");
          break;
      }
    });

    return () => {
      channel.unsubscribe();
    };
  }, []);

  return status;
}
```

### Reconnection

```typescript
const channel = supabase.channel("my-channel");

channel.subscribe((status, err) => {
  if (status === "CHANNEL_ERROR") {
    console.error("Channel error:", err);
    // Implement exponential backoff
    setTimeout(() => {
      channel.subscribe();
    }, 1000 * Math.random());
  }
});
```

### Cleanup

```typescript
// Single channel
await channel.unsubscribe();

// All channels
await supabase.removeAllChannels();
```

## Topic Naming Convention

Use consistent topic naming: `scope:entity:id`

| Pattern | Example | Use Case |
|---------|---------|----------|
| `room:{id}` | `room:123` | Chat rooms |
| `user:{id}` | `user:abc` | User notifications |
| `trip:{id}` | `trip:456` | Trip updates |
| `cursor:{room}` | `cursor:doc-123` | Cursor sharing |
| `presence:{room}` | `presence:room-456` | Online users |
