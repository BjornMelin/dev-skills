# Storage Reference

Supabase Storage patterns for file uploads, buckets, and transformations.

## Table of Contents
- [Bucket Configuration](#bucket-configuration)
- [Upload Patterns](#upload-patterns)
- [Download & Access](#download--access)
- [Image Transformations](#image-transformations)
- [Storage RLS](#storage-rls)
- [Resumable Uploads](#resumable-uploads)

## Bucket Configuration

### Create Bucket (Dashboard or Migration)

```sql
-- In migration file
insert into storage.buckets (id, name, public, file_size_limit, allowed_mime_types)
values (
  'avatars',
  'avatars',
  true,  -- Public bucket
  5242880,  -- 5MB limit
  array['image/jpeg', 'image/png', 'image/webp']
);

insert into storage.buckets (id, name, public, file_size_limit)
values (
  'documents',
  'documents',
  false,  -- Private bucket
  52428800  -- 50MB limit
);
```

### Bucket Types

| Type | Access | Use Case |
|------|--------|----------|
| Public | Direct URL, no auth | Profile pictures, public assets |
| Private | Signed URLs required | User documents, sensitive files |

## Upload Patterns

### Basic Upload

```typescript
const { data, error } = await supabase.storage
  .from("avatars")
  .upload(`${userId}/avatar.png`, file, {
    cacheControl: "3600",
    upsert: true,  // Overwrite if exists
  });

if (error) throw error;
console.log("Uploaded:", data.path);
```

### Upload with Content Type

```typescript
const { data, error } = await supabase.storage
  .from("documents")
  .upload(`${userId}/${filename}`, file, {
    contentType: file.type,
    cacheControl: "3600",
  });
```

### React Upload Component

```typescript
"use client";

import { useState } from "react";
import { getBrowserClient } from "@/lib/supabase/client";

export function FileUpload({ userId }: { userId: string }) {
  const [uploading, setUploading] = useState(false);

  async function handleUpload(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;

    const supabase = getBrowserClient();
    if (!supabase) return;

    setUploading(true);

    const fileExt = file.name.split(".").pop();
    const filePath = `${userId}/${crypto.randomUUID()}.${fileExt}`;

    const { error } = await supabase.storage
      .from("uploads")
      .upload(filePath, file);

    if (error) {
      console.error("Upload error:", error);
    }

    setUploading(false);
  }

  return (
    <input
      type="file"
      onChange={handleUpload}
      disabled={uploading}
    />
  );
}
```

### Server Action Upload

```typescript
// app/actions/upload.ts
"use server";

import { createServerSupabase } from "@/lib/supabase/server";

export async function uploadFile(formData: FormData) {
  const supabase = await createServerSupabase();
  const { data: { user } } = await supabase.auth.getUser();

  if (!user) {
    return { error: "Unauthorized" };
  }

  const file = formData.get("file") as File;
  if (!file) {
    return { error: "No file provided" };
  }

  const buffer = Buffer.from(await file.arrayBuffer());
  const filePath = `${user.id}/${file.name}`;

  const { data, error } = await supabase.storage
    .from("uploads")
    .upload(filePath, buffer, {
      contentType: file.type,
    });

  if (error) {
    return { error: error.message };
  }

  return { path: data.path };
}
```

## Download & Access

### Public Bucket URL

```typescript
// Direct public URL (no auth needed)
const { data } = supabase.storage
  .from("avatars")
  .getPublicUrl("user123/avatar.png");

console.log(data.publicUrl);
// https://<project>.supabase.co/storage/v1/object/public/avatars/user123/avatar.png
```

### Private Bucket - Signed URL

```typescript
// Time-limited signed URL
const { data, error } = await supabase.storage
  .from("documents")
  .createSignedUrl("user123/contract.pdf", 3600); // 1 hour

if (data) {
  console.log(data.signedUrl);
}
```

### Batch Signed URLs

```typescript
const { data, error } = await supabase.storage
  .from("documents")
  .createSignedUrls(
    ["file1.pdf", "file2.pdf", "file3.pdf"],
    3600
  );

// data = [{ path, signedUrl }, ...]
```

### Download File

```typescript
const { data, error } = await supabase.storage
  .from("documents")
  .download("user123/contract.pdf");

if (data) {
  // data is a Blob
  const url = URL.createObjectURL(data);
}
```

## Image Transformations

### Transform on Public URL

```typescript
const { data } = supabase.storage
  .from("avatars")
  .getPublicUrl("user123/photo.jpg", {
    transform: {
      width: 300,
      height: 300,
      resize: "cover",
      quality: 80,
    },
  });
```

### Transform Options

| Option | Values | Description |
|--------|--------|-------------|
| `width` | number | Target width in pixels |
| `height` | number | Target height in pixels |
| `resize` | `cover`, `contain`, `fill` | Resize mode |
| `quality` | 1-100 | JPEG/WebP quality |
| `format` | `origin`, `avif`, `webp` | Output format |

### URL Transform Syntax

```
?width=300&height=200&resize=cover&quality=80&format=webp
```

### Transform on Signed URL

```typescript
const { data } = await supabase.storage
  .from("documents")
  .createSignedUrl("user123/image.png", 3600, {
    transform: {
      width: 200,
      height: 200,
      resize: "contain",
    },
  });
```

## Storage RLS

### Enable RLS on Storage

```sql
-- RLS is on storage.objects table
alter table storage.objects enable row level security;
```

### Policy Patterns

#### User-Owned Files

```sql
-- Users can upload to their own folder
create policy "Users upload own files"
on storage.objects for insert
to authenticated
with check (
  bucket_id = 'uploads' and
  (storage.foldername(name))[1] = (select auth.uid())::text
);

-- Users can view own files
create policy "Users view own files"
on storage.objects for select
to authenticated
using (
  bucket_id = 'uploads' and
  (storage.foldername(name))[1] = (select auth.uid())::text
);

-- Users can delete own files
create policy "Users delete own files"
on storage.objects for delete
to authenticated
using (
  bucket_id = 'uploads' and
  (storage.foldername(name))[1] = (select auth.uid())::text
);
```

#### Public Read, Auth Write

```sql
-- Anyone can read from public bucket
create policy "Public read"
on storage.objects for select
to anon, authenticated
using (bucket_id = 'public-assets');

-- Only authenticated can upload
create policy "Auth users upload"
on storage.objects for insert
to authenticated
with check (bucket_id = 'public-assets');
```

#### Team-Based Access

```sql
-- Team members can access team files
create policy "Team members access"
on storage.objects for select
to authenticated
using (
  bucket_id = 'team-files' and
  exists (
    select 1 from public.team_members
    where team_id = (storage.foldername(name))[1]::uuid
    and user_id = (select auth.uid())
  )
);
```

### Storage Helper Functions

```sql
-- Extract first folder from path
storage.foldername(name)  -- Returns text[]

-- Example: 'user123/docs/file.pdf' → ['user123', 'docs']
```

## Resumable Uploads

### TUS Protocol Upload

```typescript
import { TusClient } from "@supabase/tus-client";

async function resumableUpload(
  file: File,
  bucketId: string,
  filePath: string
) {
  const supabase = getBrowserClient();
  if (!supabase) return;

  const { data: { session } } = await supabase.auth.getSession();
  if (!session) return;

  const tusClient = new TusClient(
    `${process.env.NEXT_PUBLIC_SUPABASE_URL}/storage/v1/upload/resumable`,
    {
      headers: {
        Authorization: `Bearer ${session.access_token}`,
      },
    }
  );

  const upload = tusClient.upload(file, {
    bucketId,
    objectPath: filePath,
    onProgress: (bytesUploaded, bytesTotal) => {
      const percentage = (bytesUploaded / bytesTotal * 100).toFixed(2);
      console.log(`Upload progress: ${percentage}%`);
    },
    onSuccess: () => {
      console.log("Upload complete!");
    },
    onError: (error) => {
      console.error("Upload error:", error);
    },
  });

  return upload;
}
```

### Upload Progress Hook

```typescript
"use client";

import { useState } from "react";

export function useFileUpload() {
  const [progress, setProgress] = useState(0);
  const [uploading, setUploading] = useState(false);

  async function upload(file: File, path: string) {
    setUploading(true);
    setProgress(0);

    // Use XMLHttpRequest for progress
    return new Promise((resolve, reject) => {
      const xhr = new XMLHttpRequest();

      xhr.upload.addEventListener("progress", (e) => {
        if (e.lengthComputable) {
          setProgress(Math.round((e.loaded / e.total) * 100));
        }
      });

      xhr.addEventListener("load", () => {
        setUploading(false);
        if (xhr.status >= 200 && xhr.status < 300) {
          resolve(JSON.parse(xhr.response));
        } else {
          reject(new Error(xhr.statusText));
        }
      });

      xhr.addEventListener("error", () => {
        setUploading(false);
        reject(new Error("Upload failed"));
      });

      const formData = new FormData();
      formData.append("file", file);

      xhr.open("POST", `/api/upload?path=${encodeURIComponent(path)}`);
      xhr.send(formData);
    });
  }

  return { upload, progress, uploading };
}
```

## File Management

### List Files

```typescript
const { data, error } = await supabase.storage
  .from("uploads")
  .list("user123/", {
    limit: 100,
    offset: 0,
    sortBy: { column: "created_at", order: "desc" },
  });

// data = [{ name, id, created_at, updated_at, metadata }, ...]
```

### Move/Rename File

```typescript
const { data, error } = await supabase.storage
  .from("uploads")
  .move("old/path/file.pdf", "new/path/file.pdf");
```

### Copy File

```typescript
const { data, error } = await supabase.storage
  .from("uploads")
  .copy("source/file.pdf", "destination/file.pdf");
```

### Delete File

```typescript
const { data, error } = await supabase.storage
  .from("uploads")
  .remove(["user123/file1.pdf", "user123/file2.pdf"]);
```
