# Supabase CLI Mastery

Complete reference for Supabase CLI workflows in development and CI/CD.

## Table of Contents
- [Installation & Setup](#installation--setup)
- [Local Development](#local-development)
- [Type Generation](#type-generation)
- [Migrations](#migrations)
- [Edge Functions](#edge-functions)
- [Database Operations](#database-operations)
- [CI/CD Integration](#cicd-integration)

## Installation & Setup

### Install CLI

```bash
# macOS
brew install supabase/tap/supabase

# npm (cross-platform)
npm install -g supabase

# Windows (scoop)
scoop bucket add supabase https://github.com/supabase/scoop-bucket.git
scoop install supabase
```

### Authentication

```bash
# Login (opens browser)
supabase login

# Link to existing project
supabase link --project-ref <project-ref>

# Get project ref from dashboard URL: app.supabase.com/project/<project-ref>
```

## Local Development

### Initialize Project

```bash
# Create supabase/ directory structure
supabase init

# Structure created:
# supabase/
# ├── config.toml          # Local config
# ├── migrations/          # SQL migrations
# ├── functions/           # Edge Functions
# └── seed.sql             # Seed data
```

### Start Local Stack

```bash
# Start local Postgres, Auth, Storage, Realtime
supabase start

# Output includes:
# - API URL: http://localhost:54321
# - DB URL: postgresql://postgres:postgres@localhost:54322/postgres
# - Studio URL: http://localhost:54323
# - Anon key and service_role key
```

### Stop Local Stack

```bash
supabase stop           # Stop containers (preserves data)
supabase stop --no-backup  # Stop and remove volumes
```

### Status Check

```bash
supabase status         # Show running services and URLs
```

## Type Generation

### Generate from Remote

```bash
# Generate types from linked project
supabase gen types typescript --project-id <project-ref> --schema public > database.types.ts

# Multiple schemas
supabase gen types typescript --project-id <ref> --schema public,auth > database.types.ts
```

### Generate from Local

```bash
# Generate from local database (requires supabase start)
supabase gen types typescript --local --schema public > database.types.ts
```

### Type Generation Best Practices

```typescript
// database.types.ts location
// Recommended: src/lib/supabase/database.types.ts

// Usage with client
import { createBrowserClient } from "@supabase/ssr";
import type { Database } from "./database.types";

const supabase = createBrowserClient<Database>(url, key);

// Now fully typed:
const { data } = await supabase.from("users").select("*");
// data is Database["public"]["Tables"]["users"]["Row"][]
```

## Migrations

### Create Migration

```bash
# Create empty migration file
supabase migration new create_users_table
# Creates: supabase/migrations/YYYYMMDDHHmmss_create_users_table.sql
```

### Migration File Format

```sql
-- supabase/migrations/20241210120000_create_users_table.sql

-- Up migration (create)
create table public.profiles (
  id uuid primary key references auth.users(id) on delete cascade,
  username text unique not null,
  avatar_url text,
  created_at timestamptz not null default now()
);

-- Enable RLS
alter table public.profiles enable row level security;

-- Create policies
create policy "Public profiles viewable"
on public.profiles for select
to authenticated
using (true);

create policy "Users update own profile"
on public.profiles for update
to authenticated
using ((select auth.uid()) = id);

-- Add index
create index profiles_username_idx on public.profiles(username);
```

### Apply Migrations

```bash
# Apply to local database
supabase db reset       # Reset and run all migrations

# Apply to remote (production)
supabase db push        # Push local migrations to remote

# Pull remote schema changes
supabase db pull        # Creates migration from remote changes
```

### Schema Diff

```bash
# Compare local schema with migrations
supabase db diff

# Generate migration from diff
supabase db diff --use-migra -f new_changes
```

### Migration Troubleshooting

```bash
# List applied migrations
supabase migration list

# Repair migration history
supabase migration repair --status applied <version>
supabase migration repair --status reverted <version>
```

## Edge Functions

### Create Function

```bash
# Create new function
supabase functions new my-function
# Creates: supabase/functions/my-function/index.ts
```

### Local Development

```bash
# Serve all functions locally
supabase functions serve

# Serve specific function
supabase functions serve my-function

# With environment variables
supabase functions serve --env-file ./supabase/.env.local
```

### Deploy Functions

```bash
# Deploy single function
supabase functions deploy my-function

# Deploy all functions
supabase functions deploy

# Deploy with JWT verification disabled (public function)
supabase functions deploy my-function --no-verify-jwt
```

### Function Secrets

```bash
# Set secrets for deployed functions
supabase secrets set MY_API_KEY=secret_value

# List secrets
supabase secrets list

# Unset secret
supabase secrets unset MY_API_KEY
```

## Database Operations

### Direct Database Access

```bash
# Connect to local database
psql postgresql://postgres:postgres@localhost:54322/postgres

# Execute SQL file
supabase db execute -f path/to/script.sql
```

### Seed Data

```bash
# Seed data location
# supabase/seed.sql

# Run seed after reset
supabase db reset  # Runs migrations then seed.sql
```

### Database Dump/Restore

```bash
# Dump schema only
supabase db dump -f schema.sql

# Dump with data
supabase db dump -f backup.sql --data-only

# Restore (use psql directly)
psql $DATABASE_URL < backup.sql
```

## CI/CD Integration

### GitHub Actions: Type Generation

```yaml
name: Generate Supabase Types
on:
  push:
    paths: ["supabase/migrations/**"]
    branches: [main]

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: supabase/setup-cli@v1
        with:
          version: latest

      - name: Generate types
        run: |
          supabase gen types typescript \
            --project-id ${{ secrets.SUPABASE_PROJECT_REF }} \
            --schema public \
            > src/lib/supabase/database.types.ts
        env:
          SUPABASE_ACCESS_TOKEN: ${{ secrets.SUPABASE_ACCESS_TOKEN }}

      - uses: peter-evans/create-pull-request@v5
        with:
          commit-message: "chore: update database types"
          title: "Update Supabase Database Types"
          branch: update-db-types
```

### GitHub Actions: Migration Deploy

```yaml
name: Deploy Migrations
on:
  push:
    paths: ["supabase/migrations/**"]
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: supabase/setup-cli@v1
        with:
          version: latest

      - name: Link project
        run: supabase link --project-ref ${{ secrets.SUPABASE_PROJECT_REF }}
        env:
          SUPABASE_ACCESS_TOKEN: ${{ secrets.SUPABASE_ACCESS_TOKEN }}

      - name: Push migrations
        run: supabase db push
        env:
          SUPABASE_ACCESS_TOKEN: ${{ secrets.SUPABASE_ACCESS_TOKEN }}
```

### GitHub Actions: Edge Function Deploy

```yaml
name: Deploy Edge Functions
on:
  push:
    paths: ["supabase/functions/**"]
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: supabase/setup-cli@v1
        with:
          version: latest

      - name: Deploy functions
        run: supabase functions deploy
        env:
          SUPABASE_ACCESS_TOKEN: ${{ secrets.SUPABASE_ACCESS_TOKEN }}
          SUPABASE_PROJECT_REF: ${{ secrets.SUPABASE_PROJECT_REF }}
```

### Required Secrets

| Secret | Description | Where to Find |
|--------|-------------|---------------|
| `SUPABASE_ACCESS_TOKEN` | Personal access token | supabase.com/dashboard/account/tokens |
| `SUPABASE_PROJECT_REF` | Project reference ID | Dashboard URL or Project Settings |

## Common Commands Reference

| Command | Description |
|---------|-------------|
| `supabase login` | Authenticate CLI |
| `supabase link --project-ref <ref>` | Link to project |
| `supabase init` | Initialize local project |
| `supabase start` | Start local stack |
| `supabase stop` | Stop local stack |
| `supabase status` | Show service status |
| `supabase gen types typescript` | Generate TypeScript types |
| `supabase migration new <name>` | Create migration |
| `supabase db reset` | Reset local database |
| `supabase db push` | Push migrations to remote |
| `supabase db pull` | Pull remote schema |
| `supabase db diff` | Show schema differences |
| `supabase functions new <name>` | Create Edge Function |
| `supabase functions serve` | Serve functions locally |
| `supabase functions deploy` | Deploy functions |
| `supabase secrets set KEY=value` | Set function secrets |
