# AI SDK Core: Embeddings & RAG Reference

Comprehensive guide to embedding generation, similarity search, reranking, and RAG patterns with the Vercel AI SDK Core.

---

## 1. Embedding Generation

### Single Value Embedding

Use `embed()` for individual text values (queries, single documents):

```typescript
import { embed } from 'ai';
import { openai } from '@ai-sdk/openai';

const { embedding, usage } = await embed({
  model: openai.embedding('text-embedding-3-small'),
  value: 'sunny day at the beach',
});

// embedding: number[] (e.g., 1536 dimensions for text-embedding-3-small)
// usage: { tokens: 10 }
console.log(embedding.length); // 1536
```

### Batch Embedding with embedMany()

Use `embedMany()` for bulk operations (preparing vector databases, batch indexing):

```typescript
import { embedMany } from 'ai';
import { openai } from '@ai-sdk/openai';

// embeddings is an array of vectors, same order as input
const { embeddings, usage } = await embedMany({
  model: openai.embedding('text-embedding-3-small'),
  values: [
    'sunny day at the beach',
    'rainy afternoon in the city',
    'snowy night in the mountains',
  ],
});

console.log(embeddings.length); // 3
console.log(usage); // { tokens: 30 }
```

### Token Usage Tracking

Both `embed()` and `embedMany()` return usage information for cost tracking:

```typescript
const { embedding, usage } = await embed({
  model: openai.embedding('text-embedding-3-small'),
  value: 'The quick brown fox jumps over the lazy dog',
});

console.log(usage.tokens); // e.g., 9 tokens
// Use for cost calculation: tokens * price_per_token
```

### Parallel Requests

Control concurrency for `embedMany()` with `maxParallelCalls`:

```typescript
const { embeddings } = await embedMany({
  model: openai.embedding('text-embedding-3-small'),
  maxParallelCalls: 2, // Process 2 batches concurrently
  values: Array.from({ length: 100 }, (_, i) => `Document ${i}`),
});

// Optimizes throughput while respecting rate limits
// Default: all requests in parallel (use with caution for large batches)
```

### Reduced Dimensions

Lower dimensional embeddings for faster similarity search and smaller storage:

```typescript
import { embed } from 'ai';
import { openai } from '@ai-sdk/openai';

// text-embedding-3-small default: 1536 dimensions
// Reduce to 512 for storage/speed optimization
const { embedding } = await embed({
  model: openai.embedding('text-embedding-3-small'),
  value: 'sunny day at the beach',
  providerOptions: {
    openai: {
      dimensions: 512, // 1/3 the size, slight accuracy tradeoff
    },
  },
});

console.log(embedding.length); // 512
```

**Dimension tradeoffs:**
- Lower dimensions: faster search, less storage, slightly lower accuracy
- Higher dimensions: better semantic accuracy, slower search, more storage
- Common reductions: 1536 → 512, 3072 → 1024

---

## 2. Embedding Models

### OpenAI

```typescript
import { openai } from '@ai-sdk/openai';

// text-embedding-3-small: 1536 dims, best cost/performance balance
const smallModel = openai.embedding('text-embedding-3-small');

// text-embedding-3-large: 3072 dims, highest accuracy
const largeModel = openai.embedding('text-embedding-3-large');

// text-embedding-ada-002: 1536 dims, legacy model
const adaModel = openai.embedding('text-embedding-ada-002');
```

**Usage example:**

```typescript
const { embedding } = await embed({
  model: openai.embedding('text-embedding-3-large'),
  value: 'Complex technical query requiring high semantic accuracy',
});
```

### Google

```typescript
import { google } from '@ai-sdk/google';

// text-embedding-004: 768 dims, multilingual support
const googleModel = google.embedding('text-embedding-004');

// With provider-specific options:
const { embedding } = await embed({
  model: googleModel,
  value: 'document text',
  providerOptions: {
    google: {
      outputDimensionality: 256, // Reduce from 768
      taskType: 'RETRIEVAL_DOCUMENT', // or 'RETRIEVAL_QUERY', 'CLASSIFICATION'
    },
  },
});
```

### Cohere

```typescript
import { cohere } from '@ai-sdk/cohere';

// embed-english-v3.0: 1024 dims, optimized for English
const englishModel = cohere.embedding('embed-english-v3.0');

// embed-multilingual-v3.0: 1024 dims, 100+ languages
const multilingualModel = cohere.embedding('embed-multilingual-v3.0');

// Light models for faster inference:
const lightModel = cohere.embedding('embed-english-light-v3.0'); // 384 dims
```

**Model Selection Guidelines:**
- **OpenAI text-embedding-3-small**: Default choice, excellent cost/performance
- **OpenAI text-embedding-3-large**: High accuracy needed (research, legal)
- **Google text-embedding-004**: Multilingual support, Google ecosystem
- **Cohere embed-multilingual-v3.0**: Best for multilingual RAG

---

## 3. Similarity Search

### cosineSimilarity() Function

Measure semantic similarity between embeddings (-1 to 1, where 1 = identical):

```typescript
import { cosineSimilarity, embedMany } from 'ai';
import { openai } from '@ai-sdk/openai';

const { embeddings } = await embedMany({
  model: openai.embedding('text-embedding-3-small'),
  values: [
    'sunny day at the beach',
    'rainy afternoon in the city',
    'beach vacation in summer',
  ],
});

const sim1 = cosineSimilarity(embeddings[0], embeddings[1]);
const sim2 = cosineSimilarity(embeddings[0], embeddings[2]);

console.log(sim1); // ~0.3 (less similar)
console.log(sim2); // ~0.8 (very similar)
```

### Finding Similar Documents

Full similarity search implementation:

```typescript
import { embed, embedMany, cosineSimilarity } from 'ai';
import { openai } from '@ai-sdk/openai';

const documents = [
  'The quick brown fox jumps over the lazy dog',
  'A fast auburn fox leaps above a sleepy canine',
  'Machine learning is a subset of artificial intelligence',
  'Deep neural networks power modern AI systems',
];

// Index documents (one-time operation)
const { embeddings: docEmbeddings } = await embedMany({
  model: openai.embedding('text-embedding-3-small'),
  values: documents,
});

// Search query
const query = 'animals jumping';
const { embedding: queryEmbedding } = await embed({
  model: openai.embedding('text-embedding-3-small'),
  value: query,
});

// Calculate similarities
const results = documents.map((doc, index) => ({
  document: doc,
  similarity: cosineSimilarity(queryEmbedding, docEmbeddings[index]),
}));

// Sort by similarity (highest first)
results.sort((a, b) => b.similarity - a.similarity);

console.log(results);
// [
//   { document: 'The quick brown fox...', similarity: 0.85 },
//   { document: 'A fast auburn fox...', similarity: 0.82 },
//   { document: 'Machine learning...', similarity: 0.12 },
//   { document: 'Deep neural networks...', similarity: 0.08 },
// ]
```

### Threshold-Based Filtering

Filter out low-relevance results:

```typescript
const MIN_SIMILARITY = 0.7; // Tune based on your domain

const relevantResults = results.filter(
  (result) => result.similarity >= MIN_SIMILARITY
);

console.log(relevantResults);
// Only returns documents with similarity >= 0.7
```

**Threshold guidelines:**
- **0.9+**: Near-duplicates
- **0.7-0.9**: Highly relevant
- **0.5-0.7**: Moderately relevant
- **<0.5**: Likely irrelevant (domain-dependent)

---

## 4. Reranking

### Basic Reranking with Cohere

Rerank documents for more accurate relevance scoring than embedding similarity:

```typescript
import { rerank } from 'ai';
import { cohere } from '@ai-sdk/cohere';

const documents = [
  'sunny day at the beach',
  'rainy afternoon in the city',
  'snowy night in the mountains',
];

const { ranking, rerankedDocuments } = await rerank({
  model: cohere.reranking('rerank-v3.5'),
  documents,
  query: 'talk about rain',
  topN: 2, // Return top 2 most relevant
});

console.log(ranking);
// [
//   { originalIndex: 1, score: 0.95, document: 'rainy afternoon in the city' },
//   { originalIndex: 0, score: 0.15, document: 'sunny day at the beach' }
// ]

console.log(rerankedDocuments);
// ['rainy afternoon in the city', 'sunny day at the beach']
```

### Structured Object Reranking

Rerank complex objects (emails, database records, JSON documents):

```typescript
import { rerank } from 'ai';
import { cohere } from '@ai-sdk/cohere';

interface Email {
  from: string;
  subject: string;
  text: string;
  timestamp: string;
}

const emails: Email[] = [
  {
    from: 'Paul Doe',
    subject: 'Follow-up: Discount Offer',
    text: 'We are happy to give you a discount of 20% on your next order.',
    timestamp: '2024-01-15',
  },
  {
    from: 'John McGill',
    subject: 'Oracle Pricing Information',
    text: 'Sorry for the delay. Here is the pricing from Oracle: $5000/month for the enterprise plan.',
    timestamp: '2024-01-16',
  },
  {
    from: 'Sarah Chen',
    subject: 'Meeting Notes',
    text: 'Following up on our discussion about database vendors and their pricing models.',
    timestamp: '2024-01-14',
  },
];

const { ranking, rerankedDocuments } = await rerank({
  model: cohere.reranking('rerank-v3.5'),
  documents: emails,
  query: 'What pricing did we get from Oracle?',
  topN: 1,
});

console.log(rerankedDocuments[0]);
// { from: 'John McGill', subject: 'Oracle Pricing...', text: '...', ... }
console.log(ranking[0].score); // 0.98 (very confident)
```

### topN Results Limiting

Efficient retrieval by limiting results:

```typescript
// Retrieve only top 3 most relevant from large candidate set
const { ranking } = await rerank({
  model: cohere.reranking('rerank-v3.5'),
  documents: largeDocumentSet, // e.g., 1000 documents
  query: 'specific technical question',
  topN: 3, // Only process and return top 3
});

// Reduces latency and cost vs returning all ranked results
```

### Scoring and Interpretation

Understanding reranking scores:

```typescript
const { ranking } = await rerank({
  model: cohere.reranking('rerank-v3.5'),
  documents: ['doc1', 'doc2', 'doc3'],
  query: 'query',
});

ranking.forEach((result) => {
  const { originalIndex, score, document } = result;

  // Score interpretation (Cohere models):
  if (score > 0.9) {
    console.log(`Highly relevant: ${document}`);
  } else if (score > 0.5) {
    console.log(`Moderately relevant: ${document}`);
  } else {
    console.log(`Low relevance: ${document}`);
  }
});
```

**Score characteristics:**
- Range: typically 0-1 (model-dependent)
- Relative: scores are comparable within a single rerank call
- Non-calibrated: 0.8 from one query ≠ 0.8 from another query
- Use thresholds empirically based on your data

---

## 5. Two-Stage Retrieval Pattern

Combine embedding similarity (fast, broad) with reranking (accurate, focused) for optimal RAG:

### Full Pipeline Implementation

```typescript
import { embed, embedMany, cosineSimilarity, rerank } from 'ai';
import { openai } from '@ai-sdk/openai';
import { cohere } from '@ai-sdk/cohere';

interface Document {
  id: string;
  content: string;
}

async function twoStageRetrieval(
  query: string,
  documents: Document[],
  candidateCount: number = 20,
  finalCount: number = 5
): Promise<Document[]> {
  // Stage 1: Embedding similarity (fast, broad retrieval)
  const { embedding: queryEmbedding } = await embed({
    model: openai.embedding('text-embedding-3-small'),
    value: query,
  });

  const { embeddings: docEmbeddings } = await embedMany({
    model: openai.embedding('text-embedding-3-small'),
    values: documents.map((doc) => doc.content),
    maxParallelCalls: 5,
  });

  // Calculate similarities
  const similarities = documents.map((doc, idx) => ({
    document: doc,
    similarity: cosineSimilarity(queryEmbedding, docEmbeddings[idx]),
  }));

  // Get top candidates
  similarities.sort((a, b) => b.similarity - a.similarity);
  const candidates = similarities.slice(0, candidateCount).map((s) => s.document);

  // Stage 2: Rerank candidates (accurate, focused)
  const { rerankedDocuments } = await rerank({
    model: cohere.reranking('rerank-v3.5'),
    documents: candidates,
    query,
    topN: finalCount,
  });

  return rerankedDocuments;
}

// Usage
const documents: Document[] = [
  { id: '1', content: 'Machine learning fundamentals...' },
  { id: '2', content: 'Neural network architectures...' },
  // ... 1000s of documents
];

const results = await twoStageRetrieval(
  'Explain backpropagation in neural networks',
  documents,
  20, // Retrieve 20 candidates with embeddings
  5   // Rerank to 5 final results
);
```

### Performance Characteristics

**Stage 1 (Embedding):** O(n) similarity calculations
- Fast: cosine similarity is computationally cheap
- Broad: reduces 10,000 docs → 20 candidates
- Recall-focused: captures all potentially relevant docs

**Stage 2 (Reranking):** O(k) where k << n
- Slower: transformer-based cross-attention
- Focused: 20 candidates → 5 results
- Precision-focused: accurate relevance scoring

**Combined:** best of both worlds
- Total time: ~100ms embedding + ~200ms reranking
- vs embedding-only: ~100ms but lower accuracy
- vs reranking-only: ~5000ms (too slow for 10k docs)

---

## 6. RAG Patterns

### Chunking Strategies

Split documents into searchable chunks:

```typescript
// Fixed-size chunking
function chunkText(text: string, chunkSize: number = 500): string[] {
  const words = text.split(/\s+/);
  const chunks: string[] = [];

  for (let i = 0; i < words.length; i += chunkSize) {
    chunks.push(words.slice(i, i + chunkSize).join(' '));
  }

  return chunks;
}

// Overlapping chunks (better context preservation)
function chunkTextWithOverlap(
  text: string,
  chunkSize: number = 500,
  overlap: number = 100
): string[] {
  const words = text.split(/\s+/);
  const chunks: string[] = [];
  const step = chunkSize - overlap;

  for (let i = 0; i < words.length; i += step) {
    chunks.push(words.slice(i, i + chunkSize).join(' '));
  }

  return chunks;
}

// Usage
const document = '...very long document...';
const chunks = chunkTextWithOverlap(document, 500, 100);

// Embed each chunk
const { embeddings } = await embedMany({
  model: openai.embedding('text-embedding-3-small'),
  values: chunks,
});
```

### Vector Database Integration Pattern

Generic pattern for vector DB operations:

```typescript
import { embedMany } from 'ai';
import { openai } from '@ai-sdk/openai';

interface VectorDBClient {
  upsert(vectors: { id: string; values: number[]; metadata: any }[]): Promise<void>;
  query(vector: number[], topK: number): Promise<any[]>;
}

async function indexDocuments(
  documents: { id: string; content: string; metadata?: any }[],
  vectorDB: VectorDBClient
): Promise<void> {
  const BATCH_SIZE = 100;

  for (let i = 0; i < documents.length; i += BATCH_SIZE) {
    const batch = documents.slice(i, i + BATCH_SIZE);

    // Generate embeddings
    const { embeddings } = await embedMany({
      model: openai.embedding('text-embedding-3-small'),
      values: batch.map((doc) => doc.content),
      maxParallelCalls: 5,
    });

    // Upsert to vector DB
    await vectorDB.upsert(
      batch.map((doc, idx) => ({
        id: doc.id,
        values: embeddings[idx],
        metadata: { content: doc.content, ...doc.metadata },
      }))
    );
  }
}

async function searchDocuments(
  query: string,
  vectorDB: VectorDBClient,
  topK: number = 5
): Promise<any[]> {
  const { embedding } = await embed({
    model: openai.embedding('text-embedding-3-small'),
    value: query,
  });

  return await vectorDB.query(embedding, topK);
}
```

### Context Injection into Prompts

Complete RAG pipeline with prompt engineering:

```typescript
import { generateText } from 'ai';
import { anthropic } from '@ai-sdk/anthropic';

async function answerQuestion(
  question: string,
  vectorDB: VectorDBClient
): Promise<string> {
  // Retrieve relevant context
  const relevantDocs = await searchDocuments(question, vectorDB, 5);

  // Format context
  const context = relevantDocs
    .map((doc, idx) => `[${idx + 1}] ${doc.metadata.content}`)
    .join('\n\n');

  // Generate answer with context
  const { text } = await generateText({
    model: anthropic('claude-3-5-sonnet-20241022'),
    prompt: `Answer the following question using only the provided context. If the context doesn't contain enough information, say so.

Context:
${context}

Question: ${question}

Answer:`,
  });

  return text;
}

// Usage
const answer = await answerQuestion(
  'What are the key benefits of RAG?',
  vectorDB
);
```

### Advanced RAG: Hybrid Search + Reranking

Combine lexical (BM25) and semantic (embeddings) search with reranking:

```typescript
async function hybridSearchWithReranking(
  query: string,
  documents: Document[],
  vectorDB: VectorDBClient
): Promise<Document[]> {
  // 1. Lexical search (BM25, full-text search)
  const lexicalResults = await lexicalSearch(query, documents, 30);

  // 2. Semantic search (embeddings)
  const semanticResults = await searchDocuments(query, vectorDB, 30);

  // 3. Merge and deduplicate
  const mergedCandidates = deduplicateDocuments([
    ...lexicalResults,
    ...semanticResults,
  ]);

  // 4. Rerank merged results
  const { rerankedDocuments } = await rerank({
    model: cohere.reranking('rerank-v3.5'),
    documents: mergedCandidates,
    query,
    topN: 5,
  });

  return rerankedDocuments;
}
```

---

## Summary

**Embedding workflow:**
1. Use `embedMany()` for batch indexing with `maxParallelCalls`
2. Use `embed()` for query-time single embeddings
3. Reduce dimensions for storage/speed optimization

**Similarity search:**
- `cosineSimilarity()` for vector comparison
- Threshold filtering for relevance control
- Sort by similarity descending

**Reranking:**
- Use for top-k candidate refinement (20 → 5)
- Supports strings and structured objects
- Higher accuracy than embeddings alone

**Two-stage retrieval:**
- Stage 1: Embeddings (fast, broad, high recall)
- Stage 2: Reranking (slow, focused, high precision)
- Best performance/accuracy tradeoff

**RAG pipeline:**
1. Chunk documents with overlap
2. Embed and index in vector DB
3. Retrieve with hybrid search
4. Rerank candidates
5. Inject context into LLM prompts
