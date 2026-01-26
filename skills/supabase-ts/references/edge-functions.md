# Edge Functions Reference

Supabase Edge Functions with Deno runtime.

## Table of Contents
- [Function Structure](#function-structure)
- [Local Development](#local-development)
- [Deployment](#deployment)
- [Authentication](#authentication)
- [Database Access](#database-access)
- [External APIs](#external-apis)
- [CORS Handling](#cors-handling)
- [Error Handling](#error-handling)

## Function Structure

### Basic Function

```typescript
// supabase/functions/hello/index.ts
import "jsr:@supabase/functions-js/edge-runtime.d.ts";

Deno.serve(async (req) => {
  const { name } = await req.json();

  return new Response(
    JSON.stringify({ message: `Hello ${name}!` }),
    { headers: { "Content-Type": "application/json" } }
  );
});
```

### Directory Structure

```
supabase/
├── functions/
│   ├── hello/
│   │   └── index.ts
│   ├── process-webhook/
│   │   └── index.ts
│   └── _shared/          # Shared code (not deployed)
│       ├── cors.ts
│       └── supabase.ts
└── config.toml
```

### Import Patterns

```typescript
// NPM packages
import OpenAI from "npm:openai@4";
import { z } from "npm:zod@3";
import Stripe from "npm:stripe@14";

// JSR packages
import "jsr:@supabase/functions-js/edge-runtime.d.ts";

// Deno standard library
import { serve } from "jsr:@std/http/server";

// Shared local code
import { corsHeaders } from "../_shared/cors.ts";
```

## Local Development

### Serve Functions Locally

```bash
# Start all functions
supabase functions serve

# Start specific function
supabase functions serve hello

# With environment variables
supabase functions serve --env-file ./supabase/.env.local
```

### Local Environment File

```env
# supabase/.env.local
OPENAI_API_KEY=sk-...
STRIPE_SECRET_KEY=sk_test_...
```

### Test Locally

```bash
# Call function
curl -i --request POST \
  http://localhost:54321/functions/v1/hello \
  --header "Authorization: Bearer $ANON_KEY" \
  --header "Content-Type: application/json" \
  --data '{"name":"World"}'
```

## Deployment

### Deploy Functions

```bash
# Deploy all functions
supabase functions deploy

# Deploy specific function
supabase functions deploy hello

# Deploy with JWT verification disabled (public)
supabase functions deploy hello --no-verify-jwt
```

### Function Secrets

```bash
# Set secrets
supabase secrets set OPENAI_API_KEY=sk-...

# List secrets
supabase secrets list

# Unset secret
supabase secrets unset OPENAI_API_KEY
```

### Production URL

```
https://<project-ref>.supabase.co/functions/v1/<function-name>
```

## Authentication

### Verify JWT (Default)

```typescript
import "jsr:@supabase/functions-js/edge-runtime.d.ts";
import { createClient } from "npm:@supabase/supabase-js@2";

Deno.serve(async (req) => {
  // Get auth header
  const authHeader = req.headers.get("Authorization");
  if (!authHeader) {
    return new Response(
      JSON.stringify({ error: "Missing authorization header" }),
      { status: 401, headers: { "Content-Type": "application/json" } }
    );
  }

  // Create client with user's token
  const supabase = createClient(
    Deno.env.get("SUPABASE_URL")!,
    Deno.env.get("SUPABASE_ANON_KEY")!,
    { global: { headers: { Authorization: authHeader } } }
  );

  // Verify user
  const { data: { user }, error } = await supabase.auth.getUser();
  if (error || !user) {
    return new Response(
      JSON.stringify({ error: "Invalid token" }),
      { status: 401, headers: { "Content-Type": "application/json" } }
    );
  }

  // User is authenticated
  return new Response(
    JSON.stringify({ userId: user.id }),
    { headers: { "Content-Type": "application/json" } }
  );
});
```

### Public Function (No Auth)

```bash
# Deploy without JWT verification
supabase functions deploy my-function --no-verify-jwt
```

```typescript
// Function handles all requests
Deno.serve(async (req) => {
  // No auth required
  return new Response("Public endpoint");
});
```

### Service Role Access

```typescript
// Use service role for admin operations
const supabaseAdmin = createClient(
  Deno.env.get("SUPABASE_URL")!,
  Deno.env.get("SUPABASE_SERVICE_ROLE_KEY")!
);

// Bypasses RLS
const { data } = await supabaseAdmin.from("users").select("*");
```

## Database Access

### With User Context (RLS)

```typescript
import { createClient } from "npm:@supabase/supabase-js@2";

Deno.serve(async (req) => {
  const authHeader = req.headers.get("Authorization")!;

  const supabase = createClient(
    Deno.env.get("SUPABASE_URL")!,
    Deno.env.get("SUPABASE_ANON_KEY")!,
    { global: { headers: { Authorization: authHeader } } }
  );

  // RLS policies apply
  const { data, error } = await supabase
    .from("items")
    .select("*");

  return Response.json({ data, error });
});
```

### Admin Access (Bypass RLS)

```typescript
const supabaseAdmin = createClient(
  Deno.env.get("SUPABASE_URL")!,
  Deno.env.get("SUPABASE_SERVICE_ROLE_KEY")!
);

// No RLS - use carefully!
const { data } = await supabaseAdmin
  .from("items")
  .insert({ title: "Admin created" });
```

## External APIs

### OpenAI

```typescript
import OpenAI from "npm:openai@4";

const openai = new OpenAI({
  apiKey: Deno.env.get("OPENAI_API_KEY"),
});

Deno.serve(async (req) => {
  const { prompt } = await req.json();

  const completion = await openai.chat.completions.create({
    model: "gpt-4",
    messages: [{ role: "user", content: prompt }],
  });

  return Response.json({
    response: completion.choices[0].message.content,
  });
});
```

### Stripe Webhooks

```typescript
import Stripe from "npm:stripe@14";

const stripe = new Stripe(Deno.env.get("STRIPE_SECRET_KEY")!, {
  apiVersion: "2023-10-16",
});

const cryptoProvider = Stripe.createSubtleCryptoProvider();

Deno.serve(async (req) => {
  const signature = req.headers.get("Stripe-Signature")!;
  const body = await req.text();

  try {
    const event = await stripe.webhooks.constructEventAsync(
      body,
      signature,
      Deno.env.get("STRIPE_WEBHOOK_SECRET")!,
      undefined,
      cryptoProvider
    );

    switch (event.type) {
      case "checkout.session.completed":
        // Handle successful payment
        break;
      case "customer.subscription.deleted":
        // Handle cancellation
        break;
    }

    return new Response(JSON.stringify({ received: true }), {
      headers: { "Content-Type": "application/json" },
    });
  } catch (err) {
    return new Response(
      JSON.stringify({ error: err.message }),
      { status: 400, headers: { "Content-Type": "application/json" } }
    );
  }
});
```

## CORS Handling

### Shared CORS Headers

```typescript
// supabase/functions/_shared/cors.ts
export const corsHeaders = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "POST, GET, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type, Authorization, x-client-info, apikey",
};
```

### Handle Preflight

```typescript
import { corsHeaders } from "../_shared/cors.ts";

Deno.serve(async (req) => {
  // Handle CORS preflight
  if (req.method === "OPTIONS") {
    return new Response(null, { headers: corsHeaders });
  }

  try {
    const data = await req.json();
    // Process request...

    return new Response(
      JSON.stringify({ success: true }),
      { headers: { ...corsHeaders, "Content-Type": "application/json" } }
    );
  } catch (error) {
    return new Response(
      JSON.stringify({ error: error.message }),
      {
        status: 500,
        headers: { ...corsHeaders, "Content-Type": "application/json" },
      }
    );
  }
});
```

### Restrict Origins

```typescript
const allowedOrigins = [
  "https://myapp.com",
  "https://staging.myapp.com",
];

function getCorsHeaders(origin: string | null) {
  const isAllowed = origin && allowedOrigins.includes(origin);
  return {
    "Access-Control-Allow-Origin": isAllowed ? origin : allowedOrigins[0],
    "Access-Control-Allow-Methods": "POST, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type, Authorization",
  };
}

Deno.serve(async (req) => {
  const origin = req.headers.get("Origin");
  const headers = getCorsHeaders(origin);

  if (req.method === "OPTIONS") {
    return new Response(null, { headers });
  }

  // ...
});
```

## Error Handling

### Structured Error Response

```typescript
interface ErrorResponse {
  error: string;
  code?: string;
  details?: unknown;
}

function errorResponse(
  message: string,
  status: number = 500,
  code?: string
): Response {
  const body: ErrorResponse = { error: message };
  if (code) body.code = code;

  return new Response(JSON.stringify(body), {
    status,
    headers: { ...corsHeaders, "Content-Type": "application/json" },
  });
}

Deno.serve(async (req) => {
  try {
    const { required_field } = await req.json();

    if (!required_field) {
      return errorResponse("required_field is missing", 400, "VALIDATION_ERROR");
    }

    // Process...
    return Response.json({ success: true });

  } catch (error) {
    console.error("Function error:", error);

    if (error instanceof SyntaxError) {
      return errorResponse("Invalid JSON body", 400, "PARSE_ERROR");
    }

    return errorResponse("Internal server error", 500, "INTERNAL_ERROR");
  }
});
```

### Logging

```typescript
Deno.serve(async (req) => {
  const requestId = crypto.randomUUID();
  console.log(`[${requestId}] Request started`, {
    method: req.method,
    url: req.url,
  });

  try {
    // Process request...
    console.log(`[${requestId}] Request completed`);
    return Response.json({ success: true });

  } catch (error) {
    console.error(`[${requestId}] Request failed:`, error);
    return Response.json({ error: error.message }, { status: 500 });
  }
});
```

## Scheduled Functions

### Cron with pg_cron

```sql
-- Enable pg_cron extension
create extension if not exists pg_cron;

-- Schedule function call
select cron.schedule(
  'daily-cleanup',
  '0 0 * * *',  -- Every day at midnight
  $$
  select
    net.http_post(
      url := 'https://<project>.supabase.co/functions/v1/cleanup',
      headers := '{"Authorization": "Bearer <service-role-key>"}'::jsonb
    )
  $$
);
```

### Using Upstash QStash

```typescript
// Trigger from application
import { Client } from "@upstash/qstash";

const qstash = new Client({ token: process.env.QSTASH_TOKEN });

await qstash.publishJSON({
  url: "https://<project>.supabase.co/functions/v1/process",
  body: { taskId: "123" },
  delay: 60, // 60 seconds
});
```
