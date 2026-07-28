---
name: cleanup-py
description: Python branch review—hard-cut, delete-first cleanup, Google docstrings.
---

# Python Branch Cleanup Skill

This skill reviews, refactors, and documents code changes in your current branch for Python repositories.
It focuses on readability, correctness, performance, and consistency with a hard-cut, deletion-first policy.

---

## Skill Overview

This skill analyzes all changes introduced in your branch and performs the following actions:

1. **Analyze Branch Changes**
   - Review uncommitted changes and outgoing commits
2. **Refactor for Readability**
   - Improve clarity, naming, structure, and modern Python usage
3. **Enhance Performance**
   - Identify safe, conservative optimization opportunities
4. **Add Documentation**
   - Apply Google-format docstrings
5. **Ensure Pattern Consistency**
   - Match the repository's existing module, API, and test patterns
6. **Validate Examples and Call Sites**
   - Keep touched code aligned with nearby examples, fixtures, and usage sites

## Default Policy

- Prefer a hard cut: keep one canonical implementation and remove fallback, compatibility, adapter, coercion, alias, and dual-shape code unless a real external boundary exists.
- Bias toward deletion: remove obsolete helpers, dead branches, stale comments, and tests that only cover abandoned shapes.
- Keep the end state small and clean: choose intention-revealing names, small single-purpose functions, and code that makes comments unnecessary.
- Validate only the current canonical contract for the branch being changed.

---

## Usage

Invoke the skill using any of the following commands:

- "Clean up my branch code"
- "Refactor the changes in my branch"
- "Review and improve my branch code"
- `/cleanup-py`

---

## What This Skill Does

### 1. Analyze Branch Changes

The skill retrieves all uncommitted changes and outgoing commits to understand:

- New files added
- Modified files
- Code additions and deletions
- Overall scope and intent of changes
- Contract or API surface changes
- Missing or stale tests
- Generated, cached, or derived files that should not be edited directly

---

### 2. Code Refactoring

#### Readability Improvements

- Replace tuples with named classes or dataclasses
- Improve variable, method, and class naming
- Extract complex logic into well-named helper methods
- Add missing type hints
- Simplify nested or complex conditionals
- Replace deprecated methods and features when the branch owns that work
- Normalize formatting to match the repository's style

#### Performance Enhancements

- Identify inefficient loops or repeated work
- Suggest appropriate data structures
- Optimize async workflows and I/O
- Remove redundant operations
- Prefer the simplest safe optimization that measurably reduces work

> Performance changes are conservative and non-breaking.

---

### 3. Documentation

Documentation follows Google-style docstrings.

#### Class Documentation

```python
class ExampleService:
    """Brief one-line description.

    Detailed explanation of the class purpose, responsibilities,
    and important behaviors.

    Supported features:

    - Feature 1
    - Feature 2
    - Feature 3
    """
```

#### Method Documentation

```python
def process_data(self, data: str, options: Optional[dict] = None) -> bool:
    """Process incoming data with optional configuration.

    Args:
        data: The input data to process.
        options: Optional configuration dictionary.

    Returns:
        True if processing succeeded, False otherwise.

    Raises:
        ValueError: If data is empty or invalid.
    """
```

#### Dataclass Parameters

```python
@dataclass
class InputParams:
    """Configuration parameters for the feature.

    Parameters:
        timeout: Request timeout in seconds.
        retry_count: Number of retry attempts.
        enable_logging: Whether to enable debug logging.
    """

    timeout: float | None = None
    retry_count: int = 3
    enable_logging: bool = False
```

---

### 4. Pattern Consistency Checks

#### Modules and APIs

- Correct inheritance and base-class usage
- Consistent constructor signatures
- Stable public method behavior
- Logging, error handling, and return-value conventions
- Alignment with adjacent modules and existing tests
- Current-shape validation only; do not preserve legacy shapes unless a real external boundary requires it.

---

### 5. Specific Implementation Patterns

#### Service or Worker Implementation

```python
class ExampleWorker:
    """Process and transform branch data for a specific workflow."""

    def __init__(self, *, api_key: str | None = None, **kwargs):
        self._api_key = api_key or os.getenv("SERVICE_API_KEY")

    async def run(self, text: str) -> str:
        # ... processing ...
        return text.strip()
```

---

#### Repository-Aware Flow

```python
def build_input(data: str) -> str:
    """Normalize input before downstream processing.

    Args:
        data: Raw input string.

    Returns:
        Normalized input string.
    """
    return data.strip()
```

#### Hard-Cut Example

When a branch introduces a new canonical shape, update producers, consumers, fixtures, and tests to use only that shape and delete the old path instead of shimming between both.

---

## Execution Flow

1. Fetch uncommitted and outgoing changes
2. Categorize files by concern
3. Analyze each file:
   - Readability
   - Performance
   - Documentation
   - Pattern consistency
4. Generate actionable recommendations
5. Apply the repository's standards and validation path

---

## Examples

### Before: Tuple Usage

```python
def get_audio_info(self) -> Tuple[int, int]:
    return (48000, 1)
```

### After: Named Class

```python
class AudioInfo:
    """Audio configuration information.

    Parameters:
        sample_rate: Sample rate in Hz.
        num_channels: Number of audio channels.
    """

    sample_rate: int
    num_channels: int

def get_audio_info(self) -> AudioInfo:
    return AudioInfo(sample_rate=48000, num_channels=1)
```

---

### Before: Missing Documentation

```python
class NewProcessor:
    def __init__(self, api_key: str, mode: str):
        self._api_key = api_key
        self._mode = mode
```

### After: Fully Documented

```python
class NewProcessor:
    """Text-processing utility using a provider-specific API.

    Supported features:
    - Input normalization
    - Streaming transformations
    - Provider customization
    - Timing metrics
    """

    def __init__(self, *, api_key: str, mode: str):
        """Initialize the service.

        Args:
            api_key: API key for authentication.
            mode: Processing mode to use.
        """
        self._api_key = api_key
        self._mode = mode
```

---

## Notes

- Hard-cut the branch to the canonical shape unless a real persisted, on-disk, wire, or public contract prevents it.
- Conservative performance changes only when they are simple, safe, and measurably useful.
- Google-style docstrings, intention-revealing names, and small functions.
- Prefer the repository's canonical formatter, linter, type checker, and tests when available.
