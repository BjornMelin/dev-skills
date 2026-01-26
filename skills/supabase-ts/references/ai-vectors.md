# AI Vectors Reference

pgvector setup, embeddings, and semantic search patterns.

## Table of Contents
- [pgvector Setup](#pgvector-setup)
- [Embedding Models](#embedding-models)
- [Vector Indexes](#vector-indexes)
- [Semantic Search](#semantic-search)
- [Hybrid Search](#hybrid-search)
- [Embedding Generation](#embedding-generation)

## pgvector Setup

### Enable Extension

```sql
-- In migration
create extension if not exists vector;
```

### Create Vector Table

```sql
create table public.documents (
  id uuid primary key default gen_random_uuid(),
  content text not null,
  embedding vector(1536),  -- Dimension matches your model
  metadata jsonb default '{}',
  created_at timestamptz not null default now()
);

alter table public.documents enable row level security;

-- RLS policy
create policy "Users view documents"
on public.documents for select
to authenticated
using (true);
```

## Embedding Models

### Common Dimensions

| Model | Dimensions | Provider |
|-------|------------|----------|
| text-embedding-3-small | 1536 | OpenAI |
| text-embedding-3-large | 3072 | OpenAI |
| text-embedding-ada-002 | 1536 | OpenAI |
| gte-small | 384 | Hugging Face |
| gte-base | 768 | Hugging Face |
| all-MiniLM-L6-v2 | 384 | Hugging Face |

### Choose Dimensions Based On

| Factor | Small (384-512) | Medium (768) | Large (1536+) |
|--------|-----------------|--------------|---------------|
| Storage | Low | Medium | High |
| Speed | Fast | Medium | Slower |
| Accuracy | Good | Better | Best |
| Use case | Simple search | General use | High precision |

## Vector Indexes

### HNSW Index (Recommended)

Best for most production use cases.

```sql
create index documents_embedding_idx
on public.documents
using hnsw (embedding vector_cosine_ops)
with (m = 16, ef_construction = 64);
```

#### HNSW Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `m` | 16 | Max connections per node (higher = better recall, more memory) |
| `ef_construction` | 64 | Size of candidate list during build (higher = better quality, slower build) |

#### Query-time Parameter

```sql
-- Set ef_search for query (higher = better recall, slower)
set hnsw.ef_search = 100;
```

### IVFFlat Index

Better for frequently updated data.

```sql
create index documents_embedding_ivf_idx
on public.documents
using ivfflat (embedding vector_cosine_ops)
with (lists = 100);
```

#### IVFFlat Parameters

| Parameter | Guideline |
|-----------|-----------|
| `lists` | sqrt(rows) for < 1M rows, rows/1000 for > 1M |

#### Query-time Parameter

```sql
-- Set probes for query (higher = better recall, slower)
set ivfflat.probes = 10;
```

### Distance Operators

| Operator | Name | Use |
|----------|------|-----|
| `<=>` | Cosine distance | Most common for text |
| `<->` | L2/Euclidean distance | Image, audio |
| `<#>` | Inner product | When vectors are normalized |

## Semantic Search

### Basic Match Function

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

### Call from TypeScript

```typescript
// Generate embedding first
const embedding = await generateEmbedding(query);

// Search
const { data, error } = await supabase.rpc("match_documents", {
  query_embedding: embedding,
  match_threshold: 0.7,
  match_count: 10,
});
```

### With Metadata Filter

```sql
create or replace function public.match_documents_filtered(
  query_embedding vector(1536),
  filter_metadata jsonb default '{}',
  match_threshold float default 0.7,
  match_count int default 10
)
returns table (
  id uuid,
  content text,
  similarity float
)
language sql
security invoker
set search_path = ''
as $$
  select
    d.id,
    d.content,
    1 - (d.embedding <=> query_embedding) as similarity
  from public.documents d
  where
    d.metadata @> filter_metadata and
    1 - (d.embedding <=> query_embedding) > match_threshold
  order by d.embedding <=> query_embedding
  limit least(match_count, 200);
$$;
```

```typescript
const { data } = await supabase.rpc("match_documents_filtered", {
  query_embedding: embedding,
  filter_metadata: { category: "tech", language: "en" },
  match_threshold: 0.7,
  match_count: 10,
});
```

## Hybrid Search

Combine keyword (full-text) and semantic search for better results.

### Setup Full-Text Search

```sql
-- Add tsvector column
alter table public.documents
add column fts tsvector
generated always as (to_tsvector('english', content)) stored;

-- Create GIN index
create index documents_fts_idx on public.documents using gin(fts);
```

### Hybrid Search Function

```sql
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
    select
      d.id,
      d.content,
      ts_rank(d.fts, websearch_to_tsquery('english', query_text)) as rank
    from public.documents d
    where d.fts @@ websearch_to_tsquery('english', query_text)
    limit match_count * 2
  ),
  semantic_results as (
    select
      d.id,
      d.content,
      1 - (d.embedding <=> query_embedding) as rank
    from public.documents d
    order by d.embedding <=> query_embedding
    limit match_count * 2
  )
  select
    coalesce(k.id, s.id) as id,
    coalesce(k.content, s.content) as content,
    (
      coalesce(k.rank, 0) * keyword_weight +
      coalesce(s.rank, 0) * semantic_weight
    ) as combined_score
  from keyword_results k
  full outer join semantic_results s on k.id = s.id
  order by combined_score desc
  limit match_count;
end;
$$;
```

### Reciprocal Rank Fusion (RRF)

```sql
create or replace function public.hybrid_search_rrf(
  query_text text,
  query_embedding vector(1536),
  match_count int default 10,
  rrf_k int default 60
)
returns table (
  id uuid,
  content text,
  rrf_score float
)
language plpgsql
security invoker
set search_path = ''
as $$
begin
  return query
  with keyword_results as (
    select
      d.id,
      d.content,
      row_number() over (
        order by ts_rank(d.fts, websearch_to_tsquery('english', query_text)) desc
      ) as rank
    from public.documents d
    where d.fts @@ websearch_to_tsquery('english', query_text)
    limit match_count * 2
  ),
  semantic_results as (
    select
      d.id,
      d.content,
      row_number() over (order by d.embedding <=> query_embedding) as rank
    from public.documents d
    limit match_count * 2
  )
  select
    coalesce(k.id, s.id) as id,
    coalesce(k.content, s.content) as content,
    (
      coalesce(1.0 / (rrf_k + k.rank), 0) +
      coalesce(1.0 / (rrf_k + s.rank), 0)
    ) as rrf_score
  from keyword_results k
  full outer join semantic_results s on k.id = s.id
  order by rrf_score desc
  limit match_count;
end;
$$;
```

## Embedding Generation

### OpenAI Embeddings

```typescript
import OpenAI from "openai";

const openai = new OpenAI();

async function generateEmbedding(text: string): Promise<number[]> {
  const response = await openai.embeddings.create({
    model: "text-embedding-3-small",
    input: text,
  });

  return response.data[0].embedding;
}
```

### Batch Embedding

```typescript
async function generateEmbeddings(texts: string[]): Promise<number[][]> {
  const response = await openai.embeddings.create({
    model: "text-embedding-3-small",
    input: texts,
  });

  return response.data.map((d) => d.embedding);
}
```

### Insert with Embedding

```typescript
async function insertDocument(content: string, metadata: object = {}) {
  const embedding = await generateEmbedding(content);

  const { data, error } = await supabase.from("documents").insert({
    content,
    embedding,
    metadata,
  });

  return { data, error };
}
```

### Edge Function for Embeddings

```typescript
// supabase/functions/embed/index.ts
import "jsr:@supabase/functions-js/edge-runtime.d.ts";
import OpenAI from "npm:openai@4";
import { createClient } from "npm:@supabase/supabase-js@2";

const openai = new OpenAI({ apiKey: Deno.env.get("OPENAI_API_KEY") });

Deno.serve(async (req) => {
  const { content, metadata } = await req.json();

  // Generate embedding
  const response = await openai.embeddings.create({
    model: "text-embedding-3-small",
    input: content,
  });

  const embedding = response.data[0].embedding;

  // Store in Supabase
  const supabase = createClient(
    Deno.env.get("SUPABASE_URL")!,
    Deno.env.get("SUPABASE_SERVICE_ROLE_KEY")!
  );

  const { data, error } = await supabase.from("documents").insert({
    content,
    embedding,
    metadata,
  });

  if (error) {
    return Response.json({ error: error.message }, { status: 500 });
  }

  return Response.json({ id: data[0].id });
});
```

## Chunking Strategies

### Fixed-Size Chunks

```typescript
function chunkText(text: string, chunkSize: number = 1000, overlap: number = 200): string[] {
  const chunks: string[] = [];
  let start = 0;

  while (start < text.length) {
    const end = Math.min(start + chunkSize, text.length);
    chunks.push(text.slice(start, end));
    start += chunkSize - overlap;
  }

  return chunks;
}
```

### Sentence-Based Chunks

```typescript
function chunkBySentences(text: string, maxChunkSize: number = 1000): string[] {
  const sentences = text.match(/[^.!?]+[.!?]+/g) || [text];
  const chunks: string[] = [];
  let currentChunk = "";

  for (const sentence of sentences) {
    if (currentChunk.length + sentence.length > maxChunkSize && currentChunk) {
      chunks.push(currentChunk.trim());
      currentChunk = "";
    }
    currentChunk += sentence;
  }

  if (currentChunk) {
    chunks.push(currentChunk.trim());
  }

  return chunks;
}
```

### Store Chunks with Source

```sql
create table public.document_chunks (
  id uuid primary key default gen_random_uuid(),
  document_id uuid not null references public.documents(id) on delete cascade,
  chunk_index int not null,
  content text not null,
  embedding vector(1536),
  created_at timestamptz not null default now()
);

create index document_chunks_embedding_idx
on public.document_chunks
using hnsw (embedding vector_cosine_ops);
```
