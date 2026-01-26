# Vercel Deployment Reference

Supabase integration with Vercel for production deployments.

## Table of Contents
- [Vercel Integration Setup](#vercel-integration-setup)
- [Environment Variables](#environment-variables)
- [Connection Pooling](#connection-pooling)
- [Next.js Configuration](#nextjs-configuration)
- [Production Checklist](#production-checklist)
- [Security Hardening](#security-hardening)

## Vercel Integration Setup

### Enable Integration

1. Go to Vercel Dashboard → Project → Settings → Integrations
2. Add Supabase integration
3. Link your Supabase project
4. Environment variables are auto-synced

### Manual Setup (Without Integration)

If not using the integration, add variables manually in Vercel Dashboard → Settings → Environment Variables.

## Environment Variables

### Auto-Synced Variables (With Integration)

The Vercel Supabase integration automatically syncs these variables:

| Variable | Purpose |
|----------|---------|
| `NEXT_PUBLIC_SUPABASE_URL` | Project API URL |
| `NEXT_PUBLIC_SUPABASE_ANON_KEY` | Anonymous/public key |
| `SUPABASE_URL` | Same as public URL (server-side) |
| `SUPABASE_ANON_KEY` | Same as public anon key |
| `SUPABASE_SERVICE_ROLE_KEY` | Service role key (server-only) |
| `POSTGRES_URL` | Pooled connection string |
| `POSTGRES_PRISMA_URL` | Prisma-specific pooled URL |
| `POSTGRES_URL_NON_POOLING` | Direct connection string |
| `POSTGRES_USER` | Database user |
| `POSTGRES_PASSWORD` | Database password |
| `POSTGRES_DATABASE` | Database name |
| `POSTGRES_HOST` | Pooler hostname |

### Required .env.local (Local Development)

```env
# Public (exposed to browser)
NEXT_PUBLIC_SUPABASE_URL=https://<project-ref>.supabase.co
NEXT_PUBLIC_SUPABASE_ANON_KEY=eyJ...

# Server-only (never expose to client)
SUPABASE_SERVICE_ROLE_KEY=eyJ...

# Direct database (for migrations)
DATABASE_URL=postgres://postgres.[ref]:[password]@aws-0-[region].pooler.supabase.com:5432/postgres
```

## Connection Pooling

### Supavisor Pooling Modes

| Port | Mode | Use Case |
|------|------|----------|
| 6543 | Transaction | Serverless functions, short-lived connections |
| 5432 | Session | Long-running connections, connection persistence |

### Serverless Configuration

```env
# Pooled connection for Vercel serverless functions
DATABASE_URL=postgres://postgres.[ref]:[password]@aws-0-[region].pooler.supabase.com:6543/postgres?pgbouncer=true

# Direct connection for migrations
DIRECT_URL=postgres://postgres.[ref]:[password]@aws-0-[region].pooler.supabase.com:5432/postgres
```

### Prisma Configuration

```prisma
// prisma/schema.prisma
datasource db {
  provider  = "postgresql"
  url       = env("DATABASE_URL")      // Pooled (6543)
  directUrl = env("DIRECT_URL")        // Direct (5432)
}
```

### Drizzle Configuration

```typescript
// drizzle.config.ts
import { defineConfig } from "drizzle-kit";

export default defineConfig({
  schema: "./src/db/schema.ts",
  out: "./drizzle",
  dialect: "postgresql",
  dbCredentials: {
    url: process.env.DATABASE_URL!,
  },
});
```

## Next.js Configuration

### Environment Variable Validation

```typescript
// src/lib/env/server.ts
import { z } from "zod";

const serverEnvSchema = z.object({
  NEXT_PUBLIC_SUPABASE_URL: z.url(),
  NEXT_PUBLIC_SUPABASE_ANON_KEY: z.string().min(1),
  SUPABASE_SERVICE_ROLE_KEY: z.string().min(1).optional(),
});

export const serverEnv = serverEnvSchema.parse(process.env);
```

### Dynamic Routes for Auth

```typescript
// next.config.ts
import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Auth routes must be dynamic (no caching)
  experimental: {
    // If using PPR
    ppr: true,
  },
};

export default nextConfig;
```

### Auth Routes Configuration

```typescript
// app/api/auth/[...supabase]/route.ts
// Ensure auth routes are dynamic
export const dynamic = "force-dynamic";
export const runtime = "nodejs"; // or "edge"
```

## Production Checklist

### Pre-Deployment

- [ ] All `NEXT_PUBLIC_*` variables are set in Vercel
- [ ] Service role key is only in server environment
- [ ] Database URL uses pooled connection (port 6543)
- [ ] Direct URL available for migrations
- [ ] RLS enabled on all tables
- [ ] Auth redirect URLs configured in Supabase Dashboard

### Supabase Dashboard Settings

1. **Authentication → URL Configuration**
   - Site URL: `https://your-domain.com`
   - Redirect URLs: Add all valid callback URLs

2. **Authentication → Providers**
   - Configure OAuth providers with production redirect URLs

3. **Database → Roles**
   - Review `authenticated` and `anon` role permissions

### Vercel Settings

1. **Project Settings → Environment Variables**
   - Verify all Supabase variables are synced

2. **Project Settings → Functions**
   - Set appropriate function timeout (default 10s)
   - Configure memory allocation if needed

## Security Hardening

### Service Role Key Protection

```typescript
// NEVER do this
const supabase = createClient(url, process.env.SUPABASE_SERVICE_ROLE_KEY!);

// CORRECT: Use in server-only contexts
import "server-only";

// Only use service role for admin operations
export async function adminOperation() {
  const supabase = createClient(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.SUPABASE_SERVICE_ROLE_KEY!,
    {
      auth: {
        autoRefreshToken: false,
        persistSession: false,
      },
    }
  );
  // Service role bypasses RLS
}
```

### Content Security Policy

```typescript
// next.config.ts
const securityHeaders = [
  {
    key: "Content-Security-Policy",
    value: `
      default-src 'self';
      script-src 'self' 'unsafe-eval' 'unsafe-inline';
      style-src 'self' 'unsafe-inline';
      connect-src 'self' https://*.supabase.co wss://*.supabase.co;
      img-src 'self' data: https://*.supabase.co;
    `.replace(/\n/g, ""),
  },
];

const nextConfig: NextConfig = {
  async headers() {
    return [
      {
        source: "/(.*)",
        headers: securityHeaders,
      },
    ];
  },
};
```

### Rate Limiting Middleware

```typescript
// middleware.ts
import { Ratelimit } from "@upstash/ratelimit";
import { Redis } from "@upstash/redis";

const ratelimit = new Ratelimit({
  redis: Redis.fromEnv(),
  limiter: Ratelimit.slidingWindow(10, "10 s"),
  analytics: true,
});

export async function middleware(request: NextRequest) {
  // Rate limit API routes
  if (request.nextUrl.pathname.startsWith("/api/")) {
    const ip = request.headers.get("x-forwarded-for") ?? "127.0.0.1";
    const { success, limit, remaining } = await ratelimit.limit(ip);

    if (!success) {
      return NextResponse.json(
        { error: "Too many requests" },
        {
          status: 429,
          headers: {
            "X-RateLimit-Limit": limit.toString(),
            "X-RateLimit-Remaining": remaining.toString(),
          },
        }
      );
    }
  }

  // Continue with Supabase auth middleware...
}
```

## Preview Deployments

### Database Branching (Enterprise)

For Supabase Pro/Enterprise with database branching:

```yaml
# vercel.json
{
  "git": {
    "deploymentEnabled": {
      "main": true,
      "preview": true
    }
  }
}
```

### Preview Environment Variables

Set preview-specific environment variables:

| Environment | Variable | Value |
|-------------|----------|-------|
| Preview | `NEXT_PUBLIC_SUPABASE_URL` | Preview project URL |
| Production | `NEXT_PUBLIC_SUPABASE_URL` | Production project URL |

## Monitoring

### Vercel Analytics Integration

```typescript
// app/layout.tsx
import { Analytics } from "@vercel/analytics/react";
import { SpeedInsights } from "@vercel/speed-insights/next";

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html>
      <body>
        {children}
        <Analytics />
        <SpeedInsights />
      </body>
    </html>
  );
}
```

### Error Tracking

```typescript
// src/lib/supabase/server.ts
import { createServerLogger } from "@/lib/telemetry/logger";

export async function createServerSupabase() {
  const logger = createServerLogger("supabase");

  try {
    // ... client creation
  } catch (error) {
    logger.error("Failed to create Supabase client", { error });
    throw error;
  }
}
```
