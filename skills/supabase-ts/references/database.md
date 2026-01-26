# Database Reference

Postgres database patterns, migrations, pgvector, and database functions.

## Table of Contents
- [Migration Conventions](#migration-conventions)
- [Table Design](#table-design)
- [Database Functions](#database-functions)
- [Extensions](#extensions)
- [pgvector Setup](#pgvector-setup)
- [Indexing Strategies](#indexing-strategies)

## Migration Conventions

### File Naming

```
supabase/migrations/YYYYMMDDHHmmss_description.sql
```

Examples:
- `20241210120000_create_users_table.sql`
- `20241210130000_add_profiles_indexes.sql`
- `20241210140000_enable_pgvector.sql`

### Migration Structure

```sql
-- 1. Extensions first
create extension if not exists "uuid-ossp";
create extension if not exists "pgcrypto";

-- 2. Types/Enums
create type public.status_type as enum ('draft', 'published', 'archived');

-- 3. Tables
create table public.posts (
  id uuid primary key default gen_random_uuid(),
  author_id uuid not null references auth.users(id) on delete cascade,
  title text not null,
  status public.status_type not null default 'draft',
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

-- 4. RLS (mandatory)
alter table public.posts enable row level security;

-- 5. Policies
create policy "Authors view own posts"
on public.posts for select
to authenticated
using ((select auth.uid()) = author_id);

-- 6. Indexes
create index posts_author_id_idx on public.posts(author_id);
create index posts_status_idx on public.posts(status) where status = 'published';

-- 7. Triggers
create trigger posts_updated_at
  before update on public.posts
  for each row execute function public.handle_updated_at();

-- 8. Comments
comment on table public.posts is 'User blog posts with status workflow';
```

## Table Design

### Standard Columns

```sql
create table public.items (
  -- Primary key: UUID preferred
  id uuid primary key default gen_random_uuid(),

  -- Foreign key to auth.users
  user_id uuid not null references auth.users(id) on delete cascade,

  -- Timestamps
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),

  -- Soft delete (optional)
  deleted_at timestamptz
);
```

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Tables | snake_case, plural | `user_profiles` |
| Columns | snake_case | `created_at` |
| Primary keys | `id` | `id uuid` |
| Foreign keys | `<table>_id` | `user_id`, `post_id` |
| Indexes | `<table>_<column>_idx` | `posts_user_id_idx` |
| Constraints | `<table>_<column>_<type>` | `posts_title_check` |

### Updated At Trigger

```sql
-- Create function once
create or replace function public.handle_updated_at()
returns trigger
language plpgsql
security invoker
set search_path = ''
as $$
begin
  new.updated_at = now();
  return new;
end;
$$;

-- Apply to tables
create trigger items_updated_at
  before update on public.items
  for each row execute function public.handle_updated_at();
```

## Database Functions

### Function Template (SECURITY INVOKER)

```sql
create or replace function public.get_user_stats(target_user_id uuid)
returns json
language plpgsql
security invoker  -- Use caller's permissions, respects RLS
set search_path = ''  -- Empty for security
as $$
declare
  result json;
begin
  select json_build_object(
    'post_count', count(*),
    'last_post', max(created_at)
  ) into result
  from public.posts
  where author_id = target_user_id;

  return result;
end;
$$;

comment on function public.get_user_stats is 'Get statistics for a user';
```

### RPC Call from Client

```typescript
const { data, error } = await supabase.rpc("get_user_stats", {
  target_user_id: userId,
});
```

### Function Security

| Setting | Use Case |
|---------|----------|
| `security invoker` | Default. Runs with caller's permissions. RLS applies. |
| `security definer` | Runs with function owner's permissions. Bypasses RLS. Use sparingly. |

```sql
-- SECURITY DEFINER example (admin operations)
create or replace function public.admin_delete_user(target_id uuid)
returns void
language plpgsql
security definer  -- Bypasses RLS
set search_path = ''
as $$
begin
  -- Only allow service_role
  if current_setting('request.jwt.claims', true)::json->>'role' != 'service_role' then
    raise exception 'Unauthorized';
  end if;

  delete from public.profiles where id = target_id;
end;
$$;
```

## Extensions

### Enable Extensions

```sql
-- In migration file
create extension if not exists "uuid-ossp";      -- UUID generation
create extension if not exists "pgcrypto";       -- Cryptographic functions
create extension if not exists "pg_trgm";        -- Trigram text search
create extension if not exists "vector";         -- pgvector for embeddings
create extension if not exists "postgis";        -- Geospatial
```

### Extension Check

```sql
-- List enabled extensions
select * from pg_extension;
```

## pgvector Setup

### Enable Extension

```sql
create extension if not exists vector;
```

### Create Table with Vector Column

```sql
create table public.documents (
  id uuid primary key default gen_random_uuid(),
  content text not null,
  embedding vector(1536),  -- OpenAI text-embedding-3-small dimension
  metadata jsonb default '{}',
  created_at timestamptz not null default now()
);

alter table public.documents enable row level security;
```

### Vector Indexes

```sql
-- HNSW Index (recommended for most cases)
-- Faster queries, slower index build
create index documents_embedding_idx
on public.documents
using hnsw (embedding vector_cosine_ops)
with (m = 16, ef_construction = 64);

-- IVFFlat Index (alternative)
-- Faster index build, good for frequent updates
create index documents_embedding_ivf_idx
on public.documents
using ivfflat (embedding vector_cosine_ops)
with (lists = 100);
```

### Index Selection

| Index Type | Best For | Trade-offs |
|------------|----------|------------|
| HNSW | Most production use cases | Slower build, more memory |
| IVFFlat | Frequent data updates | Needs retraining periodically |

### Semantic Search Function

```sql
create or replace function public.match_documents(
  query_embedding vector(1536),
  match_threshold float default 0.7,
  match_count int default 10
)
returns table (
  id uuid,
  content text,
  metadata jsonb,
  similarity float
)
language sql
security invoker
set search_path = ''
as $$
  select
    d.id,
    d.content,
    d.metadata,
    1 - (d.embedding <=> query_embedding) as similarity
  from public.documents d
  where 1 - (d.embedding <=> query_embedding) > match_threshold
  order by d.embedding <=> query_embedding
  limit least(match_count, 200);
$$;
```

### Hybrid Search (Keyword + Semantic)

```sql
-- Add full-text search column
alter table public.documents add column fts tsvector
  generated always as (to_tsvector('english', content)) stored;

create index documents_fts_idx on public.documents using gin(fts);

-- Hybrid search function
create or replace function public.hybrid_search(
  query_text text,
  query_embedding vector(1536),
  match_count int default 10,
  keyword_weight float default 0.3,
  semantic_weight float default 0.7
)
returns table (
  id uuid,
  content text,
  combined_score float
)
language plpgsql
security invoker
set search_path = ''
as $$
begin
  return query
  with keyword_results as (
    select d.id, d.content,
      ts_rank(d.fts, websearch_to_tsquery('english', query_text)) as rank
    from public.documents d
    where d.fts @@ websearch_to_tsquery('english', query_text)
    limit match_count * 2
  ),
  semantic_results as (
    select d.id, d.content,
      1 - (d.embedding <=> query_embedding) as rank
    from public.documents d
    order by d.embedding <=> query_embedding
    limit match_count * 2
  )
  select
    coalesce(k.id, s.id) as id,
    coalesce(k.content, s.content) as content,
    (coalesce(k.rank, 0) * keyword_weight +
     coalesce(s.rank, 0) * semantic_weight) as combined_score
  from keyword_results k
  full outer join semantic_results s on k.id = s.id
  order by combined_score desc
  limit match_count;
end;
$$;
```

## Indexing Strategies

### Common Index Types

```sql
-- B-tree (default): Equality and range queries
create index users_email_idx on public.users(email);

-- Partial index: Only index subset of rows
create index posts_published_idx on public.posts(created_at)
where status = 'published';

-- Composite index: Multi-column queries
create index posts_user_status_idx on public.posts(user_id, status);

-- GIN index: JSONB, arrays, full-text
create index items_metadata_idx on public.items using gin(metadata);

-- Unique index
create unique index users_email_unique on public.users(lower(email));
```

### RLS-Optimized Indexes

```sql
-- Always index foreign keys used in RLS policies
create index items_user_id_idx on public.items(user_id);

-- For team-based access patterns
create index team_members_user_id_idx on public.team_members(user_id);
create index team_members_team_id_idx on public.team_members(team_id);
```

### Index Maintenance

```sql
-- Check index usage
select
  schemaname,
  tablename,
  indexname,
  idx_scan,
  idx_tup_read
from pg_stat_user_indexes
order by idx_scan desc;

-- Rebuild index (if bloated)
reindex index concurrently public.items_user_id_idx;
```
