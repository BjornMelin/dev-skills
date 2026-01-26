# Notebook Anti-Patterns Checklist

Comprehensive checklist of common anti-patterns in ML notebooks that hurt reproducibility, maintainability, and correctness.

## Naming Anti-Patterns

| Pattern | Issue | Fix |
|---------|-------|-----|
| `Untitled.ipynb` | Default name, no context | Use descriptive names: `01_eda_customer_churn.ipynb` |
| `notebook-Copy1.ipynb` | Duplicate without purpose | Delete or rename with version/purpose |
| `final_v2_FINAL.ipynb` | Version chaos | Use git or clear versioning scheme |
| Non-POSIX characters | Portability issues | Stick to `[A-Za-z0-9._-]` |
| Spaces in filenames | CLI/scripting issues | Use underscores: `my_notebook.ipynb` |

## Execution Order Issues

### Out-of-Order Execution
```
Cell [1]: import pandas as pd
Cell [5]: df = pd.read_csv('data.csv')  # Skipped cells 2-4
Cell [3]: print(df.head())              # Uses df from cell 5
```

**Detection**: Check `execution_count` sequence in notebook JSON.

**Fix**: Restart kernel, run all cells sequentially before sharing.

### Cell Number Gaps
```
Cell [1], Cell [2], Cell [7], Cell [8]  # Missing 3-6
```

**Detection**: Look for non-sequential execution counts.

**Impact**: Indicates cells were run out of order or deleted.

### Re-executed Cells
```
Cell [1]: x = 1
Cell [15]: x = x + 1  # Run multiple times, x keeps growing
```

**Detection**: High execution counts relative to cell position.

**Fix**: Avoid modifying state; use functions with explicit inputs.

## Hidden State Problems

### Accumulated State
```python
# BAD: State accumulates across runs
results = []
for model in models:
    results.append(evaluate(model))  # List grows each cell execution
```

```python
# GOOD: Reset state each time
results = []  # Fresh start
for model in models:
    results.append(evaluate(model))
```

### Deleted Cell Dependencies
```python
# Cell was deleted, but df_cleaned is still in memory
# Other cells use df_cleaned, but can't be reproduced
print(df_cleaned.shape)  # Works now, fails on fresh kernel
```

**Fix**: Always restart kernel and run all to verify.

### Import Side Effects
```python
# BAD: Import has side effects
import matplotlib
matplotlib.use('Agg')  # Must run before pyplot import

import matplotlib.pyplot as plt  # Order matters
```

**Fix**: Group all imports at top, document order dependencies.

## Missing Modularization

### No Functions (Red Flag)
```python
# 50+ cells of inline code with no function definitions
df = pd.read_csv('data.csv')
df = df.dropna()
df['feature'] = df['a'] * df['b']
# ... 100 more lines ...
```

**Fix**: Extract repeated logic into functions.

### No Classes (Warning for Complex Projects)
```python
# Multiple related functions that share state
def preprocess(df): ...
def engineer_features(df): ...
def train_model(X, y): ...
# All operate on shared global state
```

**Fix**: Consider a Pipeline or Transformer class.

### Copy-Paste Code
```python
# Same preprocessing in multiple cells
df_train = df_train.fillna(0)
df_train = df_train.drop_duplicates()

df_test = df_test.fillna(0)  # Copy-pasted
df_test = df_test.drop_duplicates()
```

**Fix**: Create function, apply to both.

## Missing Tests

### No Assertions
```python
# Assume data is correct, no validation
df = pd.read_csv('data.csv')
model.fit(X_train, y_train)  # Hope for the best
```

**Fix**: Add sanity checks:
```python
assert df.shape[0] > 0, "Empty dataframe"
assert not df.isnull().all().any(), "All-null columns exist"
assert X_train.shape[0] == y_train.shape[0], "Shape mismatch"
```

### No Edge Case Handling
```python
# What if division by zero?
df['ratio'] = df['a'] / df['b']
```

**Fix**: Handle edge cases:
```python
df['ratio'] = df['a'] / df['b'].replace(0, np.nan)
```

## Missing Dependencies

### Undocumented Imports
```python
import pandas as pd
import numpy as np
import sklearn  # What version?
import custom_utils  # Where is this?
```

**Fix**: Create requirements.txt with pinned versions.

### System Dependencies
```python
import cv2  # Requires system OpenCV
import torch  # Requires CUDA for GPU
```

**Fix**: Document system requirements in README.

### Import Scattered Throughout
```python
# Cell 1
import pandas as pd

# Cell 15 (much later)
import seaborn as sns  # Surprise import

# Cell 30
from sklearn.ensemble import RandomForestClassifier
```

**Fix**: All imports in first cell(s).

## Data Inaccessibility

### Absolute Paths
```python
# BAD
df = pd.read_csv('/Users/john/projects/ml/data/train.csv')
```

```python
# GOOD
df = pd.read_csv('data/train.csv')  # Relative to project root
```

### Missing Data
```python
# Data file not in repo, no download instructions
df = pd.read_csv('proprietary_data.csv')
```

**Fix**: Provide data or download script.

### Hardcoded URLs Without Caching
```python
# Downloads every time, may change or disappear
df = pd.read_csv('https://example.com/data.csv')
```

**Fix**: Cache locally with versioning.

## Reproducibility Killers

### No Random Seeds
```python
# Different results every run
from sklearn.model_selection import train_test_split
X_train, X_test = train_test_split(X, y)  # No random_state
```

### Partial Seeding
```python
np.random.seed(42)  # NumPy seeded
# But sklearn, torch, random module not seeded
```

### Seeds Set Too Late
```python
# Data already shuffled before seed is set
df = df.sample(frac=1)  # Random shuffle
np.random.seed(42)  # Too late!
```

## Configuration Anti-Patterns

### Magic Numbers
```python
# What do these mean?
model = RandomForestClassifier(n_estimators=137, max_depth=8)
```

**Fix**: Use named constants or config dict:
```python
CONFIG = {
    'n_estimators': 137,  # Tuned via GridSearch on 2024-01-15
    'max_depth': 8,
}
model = RandomForestClassifier(**CONFIG)
```

### Hardcoded Hyperparameters in Multiple Cells
```python
# Cell 10
model1 = LogisticRegression(C=0.1)

# Cell 25
model2 = LogisticRegression(C=0.1)  # Same value, but have to change both
```

**Fix**: Define once, reference everywhere.

## Output Anti-Patterns

### Large Outputs in Notebook
```python
print(df)  # Prints 1M rows
df.describe()  # Huge output stored in .ipynb
```

**Fix**: Use `.head()`, limit output size.

### Plots Without Saving
```python
plt.plot(history)
plt.show()  # Only in notebook, not saved
```

**Fix**: Save plots to files:
```python
plt.savefig('figures/training_history.png', dpi=150, bbox_inches='tight')
```

### No Output Versioning
```python
# Outputs overwrite each other
model.save('model.pkl')  # Which run is this from?
```

**Fix**: Include timestamp or experiment ID:
```python
model.save(f'models/model_{timestamp}.pkl')
```
