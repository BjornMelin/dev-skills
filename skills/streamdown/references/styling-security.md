# Styling and Security

Complete guide for styling Streamdown components and securing AI-generated content.

## Tailwind CSS Setup

### Tailwind v4 (CSS-based config)

Add to your `globals.css`:

```css
@import "tailwindcss";

/* Include Streamdown component styles */
@source "../node_modules/streamdown/dist/*.js";
```

### Tailwind v3 (JS config)

Add to `tailwind.config.js`:

```js
/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './app/**/*.{js,ts,jsx,tsx}',
    './components/**/*.{js,ts,jsx,tsx}',
    './node_modules/streamdown/dist/*.js',
  ],
  // ...
};
```

## CSS Variables

Streamdown uses CSS variables for theming. Define these in your globals:

```css
:root {
  --background: 0 0% 100%;
  --foreground: 240 10% 3.9%;
  --muted: 240 4.8% 95.9%;
  --muted-foreground: 240 3.8% 46.1%;
  --border: 240 5.9% 90%;
  --primary: 240 5.9% 10%;
  --primary-foreground: 0 0% 98%;
  --destructive: 0 84.2% 60.2%;
  --radius: 0.5rem;
}

.dark {
  --background: 240 10% 3.9%;
  --foreground: 0 0% 98%;
  --muted: 240 3.7% 15.9%;
  --muted-foreground: 240 5% 64.9%;
  --border: 240 3.7% 15.9%;
  --primary: 0 0% 98%;
  --primary-foreground: 240 5.9% 10%;
  --destructive: 0 62.8% 30.6%;
}
```

### Using Variables in Mermaid

```tsx
import type { MermaidConfig } from 'streamdown';

const mermaidConfig: MermaidConfig = {
  theme: 'base',
  themeVariables: {
    fontFamily: 'Inter, system-ui, sans-serif',
    primaryColor: 'hsl(var(--muted))',
    primaryTextColor: 'hsl(var(--foreground))',
    primaryBorderColor: 'hsl(var(--border))',
    lineColor: 'hsl(var(--border))',
    secondaryColor: 'hsl(var(--background))',
    tertiaryColor: 'hsl(var(--accent))',
  },
};
```

## Data Attributes for CSS

Streamdown adds `data-streamdown` attributes for precise CSS targeting:

### Complete Selector Reference

| Selector | Element | Common Styles |
|----------|---------|---------------|
| `[data-streamdown="heading-1"]` | h1 | `text-3xl font-bold` |
| `[data-streamdown="heading-2"]` | h2 | `text-2xl font-semibold` |
| `[data-streamdown="heading-3"]` | h3 | `text-xl font-semibold` |
| `[data-streamdown="heading-4"]` | h4 | `text-lg font-medium` |
| `[data-streamdown="heading-5"]` | h5 | `text-base font-medium` |
| `[data-streamdown="heading-6"]` | h6 | `text-sm font-medium` |
| `[data-streamdown="strong"]` | strong | `font-semibold` |
| `[data-streamdown="link"]` | a | `text-primary underline` |
| `[data-streamdown="inline-code"]` | code (inline) | `bg-muted px-1 rounded` |
| `[data-streamdown="ordered-list"]` | ol | `list-decimal pl-6` |
| `[data-streamdown="unordered-list"]` | ul | `list-disc pl-6` |
| `[data-streamdown="list-item"]` | li | `my-1` |
| `[data-streamdown="blockquote"]` | blockquote | `border-l-4 pl-4 italic` |
| `[data-streamdown="horizontal-rule"]` | hr | `border-t my-4` |
| `[data-streamdown="code-block"]` | pre wrapper | `rounded-lg overflow-hidden` |
| `[data-streamdown="mermaid-block"]` | mermaid wrapper | `my-4` |
| `[data-streamdown="table-wrapper"]` | table container | `overflow-x-auto` |
| `[data-streamdown="table"]` | table | `w-full border-collapse` |
| `[data-streamdown="table-header"]` | thead | `bg-muted` |
| `[data-streamdown="table-body"]` | tbody | — |
| `[data-streamdown="table-row"]` | tr | `border-b` |
| `[data-streamdown="table-header-cell"]` | th | `px-4 py-2 text-left font-medium` |
| `[data-streamdown="table-cell"]` | td | `px-4 py-2` |
| `[data-streamdown="superscript"]` | sup | — |
| `[data-streamdown="subscript"]` | sub | — |

### Example Stylesheet

```css
/* Custom code block styling */
[data-streamdown="code-block"] {
  border: 1px solid hsl(var(--border));
  border-radius: var(--radius);
  background-color: hsl(var(--muted));
}

/* Custom blockquote */
[data-streamdown="blockquote"] {
  border-left: 3px solid hsl(var(--primary));
  padding-left: 1rem;
  font-style: italic;
  color: hsl(var(--muted-foreground));
}

/* External link indicator */
[data-streamdown="link"][href^="http"]::after {
  content: " ↗";
  font-size: 0.75em;
}

/* Striped tables */
[data-streamdown="table-body"] [data-streamdown="table-row"]:nth-child(even) {
  background-color: hsl(var(--muted) / 0.5);
}

/* Code block header */
[data-streamdown="code-block"]::before {
  content: attr(data-language);
  display: block;
  padding: 0.5rem 1rem;
  font-size: 0.75rem;
  color: hsl(var(--muted-foreground));
  border-bottom: 1px solid hsl(var(--border));
}
```

## Custom Components

Override markdown elements via the `components` prop:

### Link with External Detection

```tsx
<Streamdown
  components={{
    a: ({ href, children, ...props }) => {
      const isExternal = href?.startsWith('http');
      return (
        <a
          href={href}
          target={isExternal ? '_blank' : undefined}
          rel={isExternal ? 'noopener noreferrer' : undefined}
          className="text-primary hover:underline"
          {...props}
        >
          {children}
          {isExternal && <span className="text-xs ml-1">↗</span>}
        </a>
      );
    },
  }}
>
  {content}
</Streamdown>
```

### Custom Code Block

```tsx
<Streamdown
  components={{
    pre: ({ children, ...props }) => (
      <div className="relative group">
        <pre
          className="overflow-x-auto p-4 rounded-lg bg-muted"
          {...props}
        >
          {children}
        </pre>
      </div>
    ),
    code: ({ children, className, ...props }) => {
      // Check if it's inline code (no className) vs code block
      const isInline = !className;
      if (isInline) {
        return (
          <code className="bg-muted px-1.5 py-0.5 rounded text-sm" {...props}>
            {children}
          </code>
        );
      }
      return <code className={className} {...props}>{children}</code>;
    },
  }}
>
  {content}
</Streamdown>
```

### Custom Headings with Anchors

```tsx
import { slugify } from '@/lib/utils';

<Streamdown
  components={{
    h2: ({ children, ...props }) => {
      const text = typeof children === 'string' ? children : '';
      const id = slugify(text);
      return (
        <h2 id={id} className="group flex items-center gap-2" {...props}>
          {children}
          <a href={`#${id}`} className="opacity-0 group-hover:opacity-100">
            #
          </a>
        </h2>
      );
    },
  }}
>
  {content}
</Streamdown>
```

## Security with rehype-harden

### Default Configuration

Streamdown includes rehype-harden by default. Access via:

```tsx
import { defaultRehypePlugins } from 'streamdown';

// Default harden plugin is at:
defaultRehypePlugins.harden
```

### HardenOptions Interface

```typescript
interface HardenOptions {
  allowedImagePrefixes?: string[];  // Default: ['*'] (all allowed)
  allowedLinkPrefixes?: string[];   // Default: ['*'] (all allowed)
  allowedProtocols?: string[];      // Default: ['*'] (all allowed)
  defaultOrigin?: string;           // Origin for relative URLs
  allowDataImages?: boolean;        // Default: true
}
```

### Restricting Protocols

For AI-generated content, restrict to safe protocols:

```tsx
import { defaultRehypePlugins } from 'streamdown';
import { harden } from 'rehype-harden';

<Streamdown
  rehypePlugins={[
    defaultRehypePlugins.raw,
    defaultRehypePlugins.katex,
    [harden, {
      allowedProtocols: ['http', 'https', 'mailto'],
      // Blocks: javascript:, data:, file:, etc.
    }],
  ]}
>
  {content}
</Streamdown>
```

### Domain Allowlisting

Restrict links and images to specific domains:

```tsx
<Streamdown
  rehypePlugins={[
    defaultRehypePlugins.raw,
    defaultRehypePlugins.katex,
    [harden, {
      allowedProtocols: ['https'],
      allowedLinkPrefixes: [
        'https://your-domain.com',
        'https://docs.your-domain.com',
        'https://github.com',
      ],
      allowedImagePrefixes: [
        'https://your-cdn.com',
        'https://images.your-domain.com',
      ],
      allowDataImages: false,  // Block data: URIs for images
    }],
  ]}
>
  {content}
</Streamdown>
```

### Complete Security Configuration

Production-ready configuration for AI-generated content:

```tsx
// streamdown-config.ts
import { defaultRehypePlugins, defaultRemarkPlugins } from 'streamdown';
import type { StreamdownProps } from 'streamdown';

// Extract harden plugin for customization
const hardenRaw = defaultRehypePlugins.harden;
const hardenFn = Array.isArray(hardenRaw) ? hardenRaw[0] : hardenRaw;
const hardenDefaults = Array.isArray(hardenRaw) && hardenRaw[1]
  ? hardenRaw[1]
  : {};

export const secureRehypePlugins: StreamdownProps['rehypePlugins'] = [
  defaultRehypePlugins.raw,
  defaultRehypePlugins.katex,
  [hardenFn, {
    ...hardenDefaults,
    allowedProtocols: ['http', 'https', 'mailto'],
    allowDataImages: false,
  }],
];

export const secureRemarkPlugins: StreamdownProps['remarkPlugins'] =
  Object.values(defaultRemarkPlugins);
```

## Security Best Practices

### 1. Never Trust AI Output

AI models can generate malicious content. Always:

- Use rehype-harden with restricted protocols
- Disable `javascript:` and `data:` protocols
- Consider domain allowlisting for production

### 2. Sanitize Before Rendering

```tsx
// Bad: Direct rendering
<Streamdown>{aiResponse}</Streamdown>

// Good: With security plugins
<Streamdown rehypePlugins={secureRehypePlugins}>
  {aiResponse}
</Streamdown>
```

### 3. CSP Headers

Complement client-side sanitization with Content Security Policy:

```tsx
// next.config.js
const securityHeaders = [
  {
    key: 'Content-Security-Policy',
    value: `
      default-src 'self';
      script-src 'self' 'unsafe-eval' 'unsafe-inline';
      style-src 'self' 'unsafe-inline';
      img-src 'self' https: data:;
      font-src 'self';
    `.replace(/\n/g, ''),
  },
];
```

### 4. Image Handling

For AI-generated image URLs:

```tsx
<Streamdown
  components={{
    img: ({ src, alt, ...props }) => {
      // Validate image source
      const isAllowed = src?.startsWith('https://your-cdn.com');
      if (!isAllowed) {
        return <span className="text-muted-foreground">[Image blocked]</span>;
      }
      return (
        <img
          src={src}
          alt={alt ?? 'AI generated image'}
          loading="lazy"
          {...props}
        />
      );
    },
  }}
>
  {content}
</Streamdown>
```

### 5. Link Click Handling

Intercept and validate link clicks:

```tsx
<Streamdown
  components={{
    a: ({ href, children, ...props }) => {
      const handleClick = (e: React.MouseEvent) => {
        // Log or validate before navigation
        if (href?.includes('suspicious-pattern')) {
          e.preventDefault();
          console.warn('Blocked suspicious link:', href);
          return;
        }
      };

      return (
        <a
          href={href}
          onClick={handleClick}
          rel="noopener noreferrer"
          {...props}
        >
          {children}
        </a>
      );
    },
  }}
>
  {content}
</Streamdown>
```

## Prose Styling with Tailwind Typography

Combine Streamdown with `@tailwindcss/typography`:

```tsx
<Streamdown
  className="prose dark:prose-invert max-w-none
    prose-headings:font-semibold
    prose-a:text-primary
    prose-code:bg-muted prose-code:px-1 prose-code:rounded
    prose-pre:bg-transparent prose-pre:p-0"
>
  {content}
</Streamdown>
```

## Dark Mode Support

### CSS Variables Approach

```css
/* Automatic dark mode via CSS variables */
.dark [data-streamdown="code-block"] {
  background-color: hsl(var(--muted));
}

.dark [data-streamdown="blockquote"] {
  border-color: hsl(var(--border));
}
```

### Shiki Theme Switching

Streamdown automatically uses the second theme in dark mode:

```tsx
// [lightTheme, darkTheme]
<Streamdown shikiTheme={['github-light', 'github-dark']}>
  {content}
</Streamdown>
```

Theme pairs that work well:
- `['github-light', 'github-dark']` - Default, familiar
- `['vitesse-light', 'vitesse-dark']` - Minimal, clean
- `['one-light', 'one-dark-pro']` - VS Code style
- `['min-light', 'dracula']` - High contrast

