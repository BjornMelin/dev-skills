# RLS Cookbook

Row Level Security policy patterns for common access control scenarios.

## Table of Contents
- [RLS Fundamentals](#rls-fundamentals)
- [User-Owned Resources](#user-owned-resources)
- [Team/Organization Access](#teamorganization-access)
- [Public Read, Auth Write](#public-read-auth-write)
- [Role-Based Access](#role-based-access)
- [Storage Policies](#storage-policies)
- [Performance Optimization](#performance-optimization)

## RLS Fundamentals

### Enable RLS (Mandatory)

```sql
-- Always enable RLS on every table
alter table public.items enable row level security;
```

### Policy Structure

```sql
create policy "Policy name"
on public.table_name
for SELECT | INSERT | UPDATE | DELETE | ALL
to authenticated | anon | role_name
using (condition)         -- For SELECT, UPDATE, DELETE
with check (condition);   -- For INSERT, UPDATE
```

### Auth Functions

| Function | Returns | Use |
|----------|---------|-----|
| `auth.uid()` | UUID | Current user's ID |
| `auth.jwt()` | JSON | Full JWT claims |
| `auth.role()` | text | User's role |
| `auth.email()` | text | User's email |

### Performance Pattern

```sql
-- ALWAYS wrap auth.uid() in subquery for performance
using ((select auth.uid()) = user_id)

-- NOT this (causes repeated function calls)
using (auth.uid() = user_id)
```

## User-Owned Resources

### Basic CRUD

```sql
-- Select: Users view their own items
create policy "Users view own items"
on public.items for select
to authenticated
using ((select auth.uid()) = user_id);

-- Insert: Users create items for themselves
create policy "Users insert own items"
on public.items for insert
to authenticated
with check ((select auth.uid()) = user_id);

-- Update: Users update their own items
create policy "Users update own items"
on public.items for update
to authenticated
using ((select auth.uid()) = user_id)
with check ((select auth.uid()) = user_id);

-- Delete: Users delete their own items
create policy "Users delete own items"
on public.items for delete
to authenticated
using ((select auth.uid()) = user_id);
```

### With Soft Delete

```sql
-- Only show non-deleted items
create policy "Users view own active items"
on public.items for select
to authenticated
using (
  (select auth.uid()) = user_id
  and deleted_at is null
);

-- Soft delete = update, not delete
create policy "Users soft delete own items"
on public.items for update
to authenticated
using ((select auth.uid()) = user_id)
with check ((select auth.uid()) = user_id);
```

## Team/Organization Access

### Team Members Table

```sql
-- Team members junction table
create table public.team_members (
  team_id uuid not null references public.teams(id) on delete cascade,
  user_id uuid not null references auth.users(id) on delete cascade,
  role text not null default 'member',
  primary key (team_id, user_id)
);

alter table public.team_members enable row level security;

-- Index for RLS performance
create index team_members_user_id_idx on public.team_members(user_id);
```

### Team Resource Access

```sql
-- Users see items belonging to their teams
create policy "Team members view team items"
on public.items for select
to authenticated
using (
  exists (
    select 1 from public.team_members
    where team_id = items.team_id
    and user_id = (select auth.uid())
  )
);

-- Team members can insert items
create policy "Team members insert team items"
on public.items for insert
to authenticated
with check (
  exists (
    select 1 from public.team_members
    where team_id = items.team_id
    and user_id = (select auth.uid())
  )
);
```

### Role-Based Team Access

```sql
-- Only admins can delete team items
create policy "Team admins delete items"
on public.items for delete
to authenticated
using (
  exists (
    select 1 from public.team_members
    where team_id = items.team_id
    and user_id = (select auth.uid())
    and role = 'admin'
  )
);

-- Admins and editors can update
create policy "Team editors update items"
on public.items for update
to authenticated
using (
  exists (
    select 1 from public.team_members
    where team_id = items.team_id
    and user_id = (select auth.uid())
    and role in ('admin', 'editor')
  )
)
with check (
  exists (
    select 1 from public.team_members
    where team_id = items.team_id
    and user_id = (select auth.uid())
    and role in ('admin', 'editor')
  )
);
```

## Public Read, Auth Write

### Public Content

```sql
-- Anyone can read published posts
create policy "Public read published posts"
on public.posts for select
to anon, authenticated
using (status = 'published');

-- Authors can see their own drafts
create policy "Authors view own posts"
on public.posts for select
to authenticated
using ((select auth.uid()) = author_id);

-- Authors can insert
create policy "Authors insert posts"
on public.posts for insert
to authenticated
with check ((select auth.uid()) = author_id);

-- Authors can update own posts
create policy "Authors update own posts"
on public.posts for update
to authenticated
using ((select auth.uid()) = author_id)
with check ((select auth.uid()) = author_id);
```

### Comments System

```sql
-- Anyone can read comments on published posts
create policy "Public read comments"
on public.comments for select
to anon, authenticated
using (
  exists (
    select 1 from public.posts
    where id = comments.post_id
    and status = 'published'
  )
);

-- Authenticated users can comment
create policy "Auth users insert comments"
on public.comments for insert
to authenticated
with check ((select auth.uid()) = user_id);

-- Users can update own comments
create policy "Users update own comments"
on public.comments for update
to authenticated
using ((select auth.uid()) = user_id)
with check ((select auth.uid()) = user_id);
```

## Role-Based Access

### Admin Override

```sql
-- Admins can do everything
create policy "Admins full access"
on public.items for all
to authenticated
using (
  exists (
    select 1 from public.user_roles
    where user_id = (select auth.uid())
    and role = 'admin'
  )
)
with check (
  exists (
    select 1 from public.user_roles
    where user_id = (select auth.uid())
    and role = 'admin'
  )
);

-- Regular users limited access
create policy "Users own items"
on public.items for all
to authenticated
using ((select auth.uid()) = user_id)
with check ((select auth.uid()) = user_id);
```

### Service Role Bypass

```sql
-- For operations that need to bypass RLS
-- Use service_role key (server-only!)
const supabase = createClient(url, serviceRoleKey, {
  auth: {
    autoRefreshToken: false,
    persistSession: false,
  },
});
```

## Storage Policies

### User Avatar Folder

```sql
-- Users upload to their own folder
create policy "Users upload avatars"
on storage.objects for insert
to authenticated
with check (
  bucket_id = 'avatars' and
  (storage.foldername(name))[1] = (select auth.uid())::text
);

-- Users view their avatars
create policy "Users view own avatars"
on storage.objects for select
to authenticated
using (
  bucket_id = 'avatars' and
  (storage.foldername(name))[1] = (select auth.uid())::text
);

-- Public avatars (anyone can view)
create policy "Public avatar access"
on storage.objects for select
to anon, authenticated
using (bucket_id = 'avatars');
```

### Team Documents

```sql
-- Team members can upload
create policy "Team upload documents"
on storage.objects for insert
to authenticated
with check (
  bucket_id = 'documents' and
  exists (
    select 1 from public.team_members
    where team_id = (storage.foldername(name))[1]::uuid
    and user_id = (select auth.uid())
  )
);

-- Team members can read
create policy "Team read documents"
on storage.objects for select
to authenticated
using (
  bucket_id = 'documents' and
  exists (
    select 1 from public.team_members
    where team_id = (storage.foldername(name))[1]::uuid
    and user_id = (select auth.uid())
  )
);
```

## Performance Optimization

### Always Index Foreign Keys

```sql
-- Essential for RLS performance
create index items_user_id_idx on public.items(user_id);
create index items_team_id_idx on public.items(team_id);
create index team_members_user_id_idx on public.team_members(user_id);
create index team_members_team_id_idx on public.team_members(team_id);
```

### Avoid Function Calls in Policies

```sql
-- BAD: Function called for every row
using (auth.uid() = user_id)

-- GOOD: Subquery evaluated once
using ((select auth.uid()) = user_id)
```

### Use EXISTS for Lookups

```sql
-- GOOD: EXISTS with index
using (
  exists (
    select 1 from public.team_members
    where team_id = items.team_id
    and user_id = (select auth.uid())
  )
)

-- BAD: IN with subquery
using (
  team_id in (
    select team_id from public.team_members
    where user_id = (select auth.uid())
  )
)
```

### Materialized Role Checks

For complex role hierarchies, consider caching:

```sql
-- Materialized view for user permissions
create materialized view public.user_permissions as
select
  u.id as user_id,
  t.id as team_id,
  tm.role,
  array_agg(distinct p.permission) as permissions
from auth.users u
join public.team_members tm on tm.user_id = u.id
join public.teams t on t.id = tm.team_id
join public.role_permissions rp on rp.role = tm.role
join public.permissions p on p.id = rp.permission_id
group by u.id, t.id, tm.role;

-- Refresh periodically
refresh materialized view public.user_permissions;

-- Use in policies
using (
  exists (
    select 1 from public.user_permissions
    where user_id = (select auth.uid())
    and team_id = items.team_id
    and 'read' = any(permissions)
  )
)
```

## Testing Policies

### Test as User

```sql
-- Set role for testing
set role authenticated;
set request.jwt.claims = '{"sub": "user-uuid-here"}';

-- Test query
select * from public.items;

-- Reset
reset role;
```

### Verify Policy Coverage

```sql
-- Check which policies exist
select
  schemaname,
  tablename,
  policyname,
  permissive,
  roles,
  cmd,
  qual,
  with_check
from pg_policies
where schemaname = 'public';
```
