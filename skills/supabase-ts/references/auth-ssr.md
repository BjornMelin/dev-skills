# Auth SSR Reference

Server-side authentication patterns for Next.js App Router with @supabase/ssr.

## Table of Contents
- [Package Setup](#package-setup)
- [Client Creation Patterns](#client-creation-patterns)
- [Auth Validation](#auth-validation)
- [OAuth Providers](#oauth-providers)
- [Session Management](#session-management)
- [Protected Routes](#protected-routes)
- [Auth Callbacks](#auth-callbacks)

## Package Setup

### Install Dependencies

```bash
npm install @supabase/ssr @supabase/supabase-js
```

### Environment Variables

```env
NEXT_PUBLIC_SUPABASE_URL=https://<project-ref>.supabase.co
NEXT_PUBLIC_SUPABASE_ANON_KEY=eyJ...
```

## Client Creation Patterns

### Server Client (Route Handlers, Server Components)

```typescript
// src/lib/supabase/server.ts
import "server-only";
import { cookies } from "next/headers";
import { createServerClient, type CookieOptions } from "@supabase/ssr";
import type { Database } from "./database.types";

export async function createServerSupabase() {
  const cookieStore = await cookies();

  return createServerClient<Database>(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!,
    {
      cookies: {
        getAll() {
          return cookieStore.getAll();
        },
        setAll(cookiesToSet) {
          try {
            cookiesToSet.forEach(({ name, value, options }) => {
              cookieStore.set(name, value, options);
            });
          } catch {
            // Ignore errors when called from Server Component
          }
        },
      },
    }
  );
}
```

### Browser Client

```typescript
// src/lib/supabase/client.ts
import { createBrowserClient } from "@supabase/ssr";
import type { Database } from "./database.types";

let client: ReturnType<typeof createBrowserClient<Database>> | null = null;

export function getBrowserClient() {
  if (client) return client;
  if (typeof window === "undefined") return null;

  client = createBrowserClient<Database>(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!
  );
  return client;
}

// React hook
export function useSupabase() {
  return useMemo(getBrowserClient, []);
}
```

### Middleware Client

```typescript
// middleware.ts
import { createServerClient } from "@supabase/ssr";
import { NextResponse, type NextRequest } from "next/server";

export async function middleware(request: NextRequest) {
  let response = NextResponse.next({ request });

  const supabase = createServerClient(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!,
    {
      cookies: {
        getAll() {
          return request.cookies.getAll();
        },
        setAll(cookiesToSet) {
          cookiesToSet.forEach(({ name, value }) =>
            request.cookies.set(name, value)
          );
          response = NextResponse.next({ request });
          cookiesToSet.forEach(({ name, value, options }) =>
            response.cookies.set(name, value, options)
          );
        },
      },
    }
  );

  // Refresh session if needed
  const { data: { user } } = await supabase.auth.getUser();

  return response;
}

export const config = {
  matcher: [
    "/((?!_next/static|_next/image|favicon.ico|.*\\.(?:svg|png|jpg|jpeg|gif|webp)$).*)",
  ],
};
```

## Auth Validation

### getUser() vs getSession()

```typescript
// CORRECT: Validates JWT with Supabase auth server
const { data: { user }, error } = await supabase.auth.getUser();

// WRONG: Only reads from cookie, can be spoofed
const { data: { session } } = await supabase.auth.getSession();
```

Always use `getUser()` when:
- Checking authentication status
- Making authorization decisions
- Rendering protected content

### Server Component Auth Check

```typescript
// app/dashboard/page.tsx
import { createServerSupabase } from "@/lib/supabase/server";
import { redirect } from "next/navigation";

export default async function DashboardPage() {
  const supabase = await createServerSupabase();
  const { data: { user } } = await supabase.auth.getUser();

  if (!user) {
    redirect("/login");
  }

  return <Dashboard userId={user.id} />;
}
```

### Route Handler Auth Check

```typescript
// app/api/profile/route.ts
import { createServerSupabase } from "@/lib/supabase/server";
import { NextResponse } from "next/server";

export async function GET() {
  const supabase = await createServerSupabase();
  const { data: { user } } = await supabase.auth.getUser();

  if (!user) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }

  const { data: profile } = await supabase
    .from("profiles")
    .select("*")
    .eq("id", user.id)
    .single();

  return NextResponse.json(profile);
}
```

## OAuth Providers

### Google OAuth

```typescript
// Sign in with Google
async function signInWithGoogle() {
  const supabase = getBrowserClient();
  if (!supabase) return;

  const { error } = await supabase.auth.signInWithOAuth({
    provider: "google",
    options: {
      redirectTo: `${window.location.origin}/auth/callback`,
      queryParams: {
        access_type: "offline",
        prompt: "consent",
      },
    },
  });
}
```

### GitHub OAuth

```typescript
async function signInWithGitHub() {
  const supabase = getBrowserClient();
  if (!supabase) return;

  const { error } = await supabase.auth.signInWithOAuth({
    provider: "github",
    options: {
      redirectTo: `${window.location.origin}/auth/callback`,
      scopes: "read:user user:email",
    },
  });
}
```

### OAuth Configuration

In Supabase Dashboard → Authentication → Providers:

| Provider | Redirect URL |
|----------|--------------|
| Google | `https://<project-ref>.supabase.co/auth/v1/callback` |
| GitHub | `https://<project-ref>.supabase.co/auth/v1/callback` |

## Session Management

### Email/Password Sign Up

```typescript
async function signUp(email: string, password: string) {
  const supabase = getBrowserClient();
  if (!supabase) return { error: new Error("Client unavailable") };

  const { data, error } = await supabase.auth.signUp({
    email,
    password,
    options: {
      emailRedirectTo: `${window.location.origin}/auth/callback`,
    },
  });

  return { data, error };
}
```

### Email/Password Sign In

```typescript
async function signIn(email: string, password: string) {
  const supabase = getBrowserClient();
  if (!supabase) return { error: new Error("Client unavailable") };

  const { data, error } = await supabase.auth.signInWithPassword({
    email,
    password,
  });

  return { data, error };
}
```

### Sign Out

```typescript
async function signOut() {
  const supabase = getBrowserClient();
  if (!supabase) return;

  await supabase.auth.signOut();
  window.location.href = "/";
}
```

### Password Reset

```typescript
// Request reset email
async function resetPassword(email: string) {
  const supabase = getBrowserClient();
  if (!supabase) return;

  const { error } = await supabase.auth.resetPasswordForEmail(email, {
    redirectTo: `${window.location.origin}/auth/reset-password`,
  });

  return { error };
}

// Update password (after clicking email link)
async function updatePassword(newPassword: string) {
  const supabase = getBrowserClient();
  if (!supabase) return;

  const { error } = await supabase.auth.updateUser({
    password: newPassword,
  });

  return { error };
}
```

## Protected Routes

### Middleware Protection

```typescript
// middleware.ts
export async function middleware(request: NextRequest) {
  const response = NextResponse.next({ request });

  const supabase = createServerClient(/* ... */);
  const { data: { user } } = await supabase.auth.getUser();

  // Protect dashboard routes
  if (!user && request.nextUrl.pathname.startsWith("/dashboard")) {
    const url = request.nextUrl.clone();
    url.pathname = "/login";
    url.searchParams.set("redirect", request.nextUrl.pathname);
    return NextResponse.redirect(url);
  }

  // Redirect authenticated users from auth pages
  if (user && (request.nextUrl.pathname === "/login" ||
               request.nextUrl.pathname === "/signup")) {
    return NextResponse.redirect(new URL("/dashboard", request.url));
  }

  return response;
}
```

### Layout-Level Protection

```typescript
// app/(protected)/layout.tsx
import { createServerSupabase } from "@/lib/supabase/server";
import { redirect } from "next/navigation";

export default async function ProtectedLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const supabase = await createServerSupabase();
  const { data: { user } } = await supabase.auth.getUser();

  if (!user) {
    redirect("/login");
  }

  return <>{children}</>;
}
```

## Auth Callbacks

### OAuth Callback Route

```typescript
// app/auth/callback/route.ts
import { createServerSupabase } from "@/lib/supabase/server";
import { NextResponse } from "next/server";

export async function GET(request: Request) {
  const requestUrl = new URL(request.url);
  const code = requestUrl.searchParams.get("code");
  const next = requestUrl.searchParams.get("next") ?? "/dashboard";

  if (code) {
    const supabase = await createServerSupabase();
    const { error } = await supabase.auth.exchangeCodeForSession(code);

    if (!error) {
      return NextResponse.redirect(new URL(next, requestUrl.origin));
    }
  }

  return NextResponse.redirect(new URL("/auth/error", requestUrl.origin));
}
```

### Email Confirmation Callback

```typescript
// app/auth/confirm/route.ts
import { createServerSupabase } from "@/lib/supabase/server";
import { type EmailOtpType } from "@supabase/supabase-js";
import { NextResponse } from "next/server";

export async function GET(request: Request) {
  const requestUrl = new URL(request.url);
  const token_hash = requestUrl.searchParams.get("token_hash");
  const type = requestUrl.searchParams.get("type") as EmailOtpType | null;
  const next = requestUrl.searchParams.get("next") ?? "/";

  if (token_hash && type) {
    const supabase = await createServerSupabase();
    const { error } = await supabase.auth.verifyOtp({
      type,
      token_hash,
    });

    if (!error) {
      return NextResponse.redirect(new URL(next, requestUrl.origin));
    }
  }

  return NextResponse.redirect(new URL("/auth/error", requestUrl.origin));
}
```

## Auth State Listener (Client)

```typescript
"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { getBrowserClient } from "@/lib/supabase/client";

export function AuthListener({ children }: { children: React.ReactNode }) {
  const router = useRouter();

  useEffect(() => {
    const supabase = getBrowserClient();
    if (!supabase) return;

    const {
      data: { subscription },
    } = supabase.auth.onAuthStateChange((event, session) => {
      if (event === "SIGNED_OUT") {
        router.push("/login");
      }
      if (event === "SIGNED_IN") {
        router.refresh();
      }
    });

    return () => subscription.unsubscribe();
  }, [router]);

  return <>{children}</>;
}
```
