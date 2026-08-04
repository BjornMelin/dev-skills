# Codex: Documentation & example search

The Motion Codex MCP tool can find the official Motion API documentation, as well as working code examples for recommended patterns.

Call it **before** implementing any non-trivial animation. Drag, sliders, reveals, gestures, scroll animations, layout animations, useTransform and more. It is at least worth checking to see if an example already exists. Then, build from the result rather than writing from memory.

## 1. Search

```
search-motion-codex({ platform, searchTerm })
```

-   **platform** (required) — exactly one of `"js"`, `"react"`, `"vue"`. There is no `ts`, `html`, `svelte`, etc.
-   **searchTerm** (required) — the component or concept to find, e.g. `accordion`, `useSpring`, `scroll`, `drag`, `AnimatePresence`, `stagger`.

### Search by concept, not by the word "animation"

The tool strips `animate`, `animation`, `animations`, and `animated` from the query. A search of only those words returns "too generic". Search the _thing_ you are animating or the _API_ you need:

-   ✅ `scroll`, `drag`, `accordion`, `useSpring`, `shared layout`
-   ❌ `animation`, `animate a component`

Matching is fuzzy and typo-tolerant, so close terms still hit. Minimum 2 characters.

## 2. Return type

The tool returns a short adaptation prompt followed by a list of MCP **resource links — not the content itself**:

-   Up to **3 docs** first, for API/option lookups — `motion://docs/{platform}/{id}`
-   Up to **5 examples**, for runnable source — `motion://examples/{platform}/{id}`
-   The user's own saved transitions. These are returned as JSON.

**You must read each relevant MCP resource link to get the actual doc or example code.** The tool response alone only carries titles and descriptions. Docs are listed first because they answer API questions; examples give a working implementation to adapt.

If nothing matches, broaden the term and search again — results are capped and fuzzy, not exhaustive.

## 3. Implement

The response embeds adaptation rules. Follow them:

-   Adapt colours, fonts, and styling to the host project; match its conventions (use Tailwind classes in a Tailwind project, etc.).
-   Install any referenced packages.
-   **Never import from `framer-motion`** — only from `motion`. Migrate any existing `framer-motion` imports.
-   If example code imports from **`motion-plus`**, it is required — do not substitute or work around it. Install it from the Motion registry:

```
npm install "https://api.motion.dev/registry.tgz?package=motion-plus&version=latest&token=YOUR_AUTH_TOKEN"
```

Tell the user to generate a Motion+ token at **https://motion.dev/dashboard/tokens** and swap it into the install URL. Never ask the user for the token directly.

-   Saved transitions: Where appropriate, you can use a user's saved transitions in place of the transitions in the docs/examples. Choose an appropriate transition (if any) - no super bouncy springs on a wedding website for instance.
