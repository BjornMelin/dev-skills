# Workflow Patterns Reference

Building blocks for reliable AI agent workflows.

> **Note**: ToolLoopAgent uses `stopWhen: stepCountIs(20)` by default. The patterns below use `generateText`/`generateObject` directly for simpler workflows. Use ToolLoopAgent for complex multi-step agents.

## Pattern Selection Guide

| Pattern | Use Case | Flexibility | Control |
|---------|----------|-------------|---------|
| Sequential | Pipelines, transformations | Low | High |
| Parallel | Independent tasks | Medium | High |
| Routing | Input-dependent paths | Medium | Medium |
| Orchestrator-Worker | Complex multi-expert tasks | High | Medium |
| Evaluator-Optimizer | Quality-critical outputs | Low | High |

**Start simple**: Add complexity only when needed.

## Sequential Processing (Chains)

Steps executed in order, each output becomes next input:

```typescript
import { generateText, generateObject } from 'ai';
import { z } from 'zod';

async function generateMarketingCopy(input: string) {
  const model = 'openai/gpt-4o';

  // Step 1: Generate copy
  const { text: copy } = await generateText({
    model,
    prompt: `Write persuasive marketing copy for: ${input}`,
  });

  // Step 2: Quality check
  const { object: qualityMetrics } = await generateObject({
    model,
    schema: z.object({
      hasCallToAction: z.boolean(),
      emotionalAppeal: z.number().min(1).max(10),
      clarity: z.number().min(1).max(10),
    }),
    prompt: `Evaluate this marketing copy: ${copy}`,
  });

  // Step 3: Improve if needed
  if (!qualityMetrics.hasCallToAction || qualityMetrics.emotionalAppeal < 7) {
    const { text: improvedCopy } = await generateText({
      model,
      prompt: `Rewrite with improvements: ${copy}`,
    });
    return { copy: improvedCopy, qualityMetrics };
  }

  return { copy, qualityMetrics };
}
```

## Routing

Model-driven path selection based on input:

```typescript
async function handleCustomerQuery(query: string) {
  // Step 1: Classify
  const { object: classification } = await generateObject({
    model: 'openai/gpt-4o',
    schema: z.object({
      type: z.enum(['general', 'refund', 'technical']),
      complexity: z.enum(['simple', 'complex']),
    }),
    prompt: `Classify this query: ${query}`,
  });

  // Step 2: Route based on classification
  const { text: response } = await generateText({
    model: classification.complexity === 'simple'
      ? 'openai/gpt-4o-mini'
      : 'openai/o4-mini',
    system: {
      general: 'You are a customer service agent.',
      refund: 'You specialize in refund requests.',
      technical: 'You are a technical support specialist.',
    }[classification.type],
    prompt: query,
  });

  return { response, classification };
}
```

## Parallel Processing

Independent tasks run simultaneously:

```typescript
async function parallelCodeReview(code: string) {
  const [securityReview, performanceReview, maintainabilityReview] =
    await Promise.all([
      generateObject({
        model: 'openai/gpt-4o',
        system: 'You are a security expert.',
        schema: z.object({
          vulnerabilities: z.array(z.string()),
          riskLevel: z.enum(['low', 'medium', 'high']),
        }),
        prompt: `Review for security: ${code}`,
      }),

      generateObject({
        model: 'openai/gpt-4o',
        system: 'You are a performance expert.',
        schema: z.object({
          issues: z.array(z.string()),
          impact: z.enum(['low', 'medium', 'high']),
        }),
        prompt: `Review for performance: ${code}`,
      }),

      generateObject({
        model: 'openai/gpt-4o',
        system: 'You are a code quality expert.',
        schema: z.object({
          concerns: z.array(z.string()),
          qualityScore: z.number().min(1).max(10),
        }),
        prompt: `Review for quality: ${code}`,
      }),
    ]);

  // Aggregate results
  const { text: summary } = await generateText({
    model: 'openai/gpt-4o',
    system: 'You are a technical lead.',
    prompt: `Synthesize these reviews: ${JSON.stringify([
      securityReview.object,
      performanceReview.object,
      maintainabilityReview.object,
    ])}`,
  });

  return { reviews: [securityReview, performanceReview, maintainabilityReview], summary };
}
```

## Orchestrator-Worker

Primary model coordinates specialized workers:

```typescript
async function implementFeature(featureRequest: string) {
  // Orchestrator: Plan the implementation
  const { object: plan } = await generateObject({
    model: 'anthropic/claude-sonnet-4.5',
    schema: z.object({
      files: z.array(z.object({
        purpose: z.string(),
        filePath: z.string(),
        changeType: z.enum(['create', 'modify', 'delete']),
      })),
      estimatedComplexity: z.enum(['low', 'medium', 'high']),
    }),
    system: 'You are a senior software architect.',
    prompt: `Plan implementation for: ${featureRequest}`,
  });

  // Workers: Execute planned changes
  const fileChanges = await Promise.all(
    plan.files.map(async file => {
      const workerSystem = {
        create: 'You implement new files following best practices.',
        modify: 'You modify existing code maintaining consistency.',
        delete: 'You safely remove code avoiding breaking changes.',
      }[file.changeType];

      const { object: change } = await generateObject({
        model: 'openai/gpt-4o',
        schema: z.object({
          explanation: z.string(),
          code: z.string(),
        }),
        system: workerSystem,
        prompt: `Implement ${file.filePath}: ${file.purpose}`,
      });

      return { file, implementation: change };
    })
  );

  return { plan, changes: fileChanges };
}
```

## Evaluator-Optimizer

Iterative quality improvement loop:

```typescript
async function translateWithFeedback(text: string, targetLanguage: string) {
  let currentTranslation = '';
  let iterations = 0;
  const MAX_ITERATIONS = 3;

  // Initial translation
  const { text: translation } = await generateText({
    model: 'anthropic/claude-sonnet-4.5',
    system: 'You are an expert literary translator.',
    prompt: `Translate to ${targetLanguage}: ${text}`,
  });

  currentTranslation = translation;

  // Evaluation-optimization loop
  while (iterations < MAX_ITERATIONS) {
    // Evaluate
    const { object: evaluation } = await generateObject({
      model: 'anthropic/claude-sonnet-4.5',
      schema: z.object({
        qualityScore: z.number().min(1).max(10),
        preservesTone: z.boolean(),
        culturallyAccurate: z.boolean(),
        specificIssues: z.array(z.string()),
      }),
      system: 'You evaluate literary translations.',
      prompt: `Evaluate:
Original: ${text}
Translation: ${currentTranslation}`,
    });

    // Check if quality meets threshold
    if (evaluation.qualityScore >= 8 && evaluation.preservesTone && evaluation.culturallyAccurate) {
      break;
    }

    // Improve based on feedback
    const { text: improved } = await generateText({
      model: 'anthropic/claude-sonnet-4.5',
      system: 'You are an expert literary translator.',
      prompt: `Improve this translation:
Issues: ${evaluation.specificIssues.join(', ')}
Original: ${text}
Current: ${currentTranslation}`,
    });

    currentTranslation = improved;
    iterations++;
  }

  return { translation: currentTranslation, iterations };
}
```

## Combining Patterns

Real-world agents often combine multiple patterns:

```typescript
async function processDocument(document: string) {
  // 1. Route based on document type
  const { object: docType } = await generateObject({
    model: 'openai/gpt-4o',
    schema: z.object({ type: z.enum(['contract', 'report', 'email']) }),
    prompt: `Classify document type: ${document}`,
  });

  // 2. Parallel extraction based on type
  if (docType.type === 'contract') {
    const [parties, terms, risks] = await Promise.all([
      extractParties(document),
      extractTerms(document),
      analyzeRisks(document),
    ]);

    // 3. Evaluate and improve summary
    return await evaluateAndImprove({ parties, terms, risks });
  }

  // ... handle other document types
}
```

## Best Practices

1. **Start simple**: Use single agent + tools before workflows
2. **Minimize handoffs**: Each step adds latency and error potential
3. **Error handling**: Implement retries and fallbacks
4. **Cost awareness**: Complex workflows = more API calls
5. **Test components**: Unit test each step independently
