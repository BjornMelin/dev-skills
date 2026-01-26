# ML Notebook Workflow Guide

Standard structure and organization patterns for production-quality ML notebooks.

## Standard ML Workflow Sections

A well-organized ML notebook follows a logical flow from data to insights. Use markdown headers to clearly delineate sections.

```
## 1. Setup
## 2. Data Loading
## 3. Exploratory Data Analysis (EDA)
## 4. Data Preprocessing
## 5. Feature Engineering
## 6. Train/Test Split
## 7. Model Definition
## 8. Training
## 9. Evaluation
## 10. Error Analysis
## 11. Save Artifacts
## 12. Conclusions
```

---

## Section Details

### 1. Setup

**Purpose**: Initialize environment, set seeds, import libraries.

**Contents**:
- All imports (stdlib, third-party, local)
- Random seed setting
- Display/plotting configuration
- Environment capture

**Template**:
```python
## 1. Setup

# Standard library
import os
import sys
import json
from pathlib import Path
from datetime import datetime

# Data handling
import numpy as np
import pandas as pd

# Visualization
import matplotlib.pyplot as plt
import seaborn as sns

# ML
from sklearn.model_selection import train_test_split
from sklearn.preprocessing import StandardScaler
from sklearn.metrics import accuracy_score, classification_report

# Configuration
SEED = 42
np.random.seed(SEED)

# Display settings
pd.set_option('display.max_columns', 50)
plt.style.use('seaborn-v0_8-whitegrid')
%matplotlib inline

# Environment info
print(f"Python: {sys.version}")
print(f"NumPy: {np.__version__}")
print(f"Pandas: {pd.__version__}")
```

**Checklist**:
- [ ] All imports at top
- [ ] Seeds set before any randomness
- [ ] Versions logged
- [ ] Config variables defined

---

### 2. Data Loading

**Purpose**: Load raw data, document source and version.

**Contents**:
- Data source documentation
- Loading code
- Basic shape/type verification

**Template**:
```python
## 2. Data Loading

# Data source: [describe source]
# Version: [version/date]
# Download: [URL or path]

DATA_PATH = Path('data/raw')
df = pd.read_csv(DATA_PATH / 'dataset.csv')

print(f"Loaded {len(df):,} rows, {len(df.columns)} columns")
print(f"Memory usage: {df.memory_usage(deep=True).sum() / 1e6:.2f} MB")
```

**Checklist**:
- [ ] Relative paths used
- [ ] Source documented
- [ ] Shape verified
- [ ] Data type check

---

### 3. Exploratory Data Analysis (EDA)

**Purpose**: Understand data characteristics before modeling.

**Contents**:
- Basic statistics
- Missing value analysis
- Distribution analysis
- Correlation analysis
- Target variable analysis

**Template**:
```python
## 3. Exploratory Data Analysis

### 3.1 Basic Info
print(df.info())
df.describe()

### 3.2 Missing Values
missing = df.isnull().sum()
missing_pct = (missing / len(df) * 100).round(2)
pd.DataFrame({'missing': missing, 'pct': missing_pct}).query('missing > 0')

### 3.3 Target Distribution
df['target'].value_counts(normalize=True).plot(kind='bar')
plt.title('Target Distribution')
plt.show()

### 3.4 Feature Distributions
df.hist(figsize=(15, 10), bins=30)
plt.tight_layout()
plt.show()

### 3.5 Correlations
plt.figure(figsize=(12, 8))
sns.heatmap(df.corr(), annot=True, cmap='coolwarm', center=0)
plt.title('Feature Correlations')
plt.show()
```

**Checklist**:
- [ ] Shape and types documented
- [ ] Missing values identified
- [ ] Target distribution checked (class imbalance?)
- [ ] Feature distributions visualized
- [ ] Correlations analyzed
- [ ] Outliers identified

---

### 4. Data Preprocessing

**Purpose**: Clean and prepare data for modeling.

**Contents**:
- Missing value handling
- Outlier treatment
- Data type conversions
- Encoding categorical variables

**Template**:
```python
## 4. Data Preprocessing

### 4.1 Handle Missing Values
# Document strategy for each column
df['numeric_col'] = df['numeric_col'].fillna(df['numeric_col'].median())
df['categorical_col'] = df['categorical_col'].fillna('Unknown')

### 4.2 Handle Outliers
# Using IQR method for numeric columns
Q1, Q3 = df['value'].quantile([0.25, 0.75])
IQR = Q3 - Q1
df = df[(df['value'] >= Q1 - 1.5*IQR) & (df['value'] <= Q3 + 1.5*IQR)]

### 4.3 Encode Categoricals
# One-hot encoding for low cardinality
df = pd.get_dummies(df, columns=['category'], drop_first=True)

print(f"After preprocessing: {df.shape}")
```

**Checklist**:
- [ ] Missing value strategy documented
- [ ] Outlier handling documented
- [ ] Encoding strategy documented
- [ ] No data leakage (fit on train only - see Section 6)

---

### 5. Feature Engineering

**Purpose**: Create features that improve model performance.

**Contents**:
- New feature creation
- Feature transformations
- Feature selection rationale

**Template**:
```python
## 5. Feature Engineering

### 5.1 Create New Features
df['feature_ratio'] = df['feature_a'] / (df['feature_b'] + 1)
df['feature_interaction'] = df['feature_a'] * df['feature_b']

### 5.2 Transformations
df['log_value'] = np.log1p(df['value'])

### 5.3 Date Features (if applicable)
df['day_of_week'] = df['date'].dt.dayofweek
df['month'] = df['date'].dt.month
df['is_weekend'] = df['day_of_week'].isin([5, 6]).astype(int)

print(f"Feature count: {len(df.columns)}")
```

**Checklist**:
- [ ] Features have clear rationale
- [ ] Transformations documented
- [ ] Feature creation before split (if using only input features)
- [ ] Target-based features created after split (if any)

---

### 6. Train/Test Split

**Purpose**: Create holdout set for unbiased evaluation.

**Contents**:
- Split with stratification (if needed)
- Fit scalers/encoders on train only
- Verify no leakage

**Template**:
```python
## 6. Train/Test Split

# Define features and target
FEATURE_COLS = [col for col in df.columns if col != 'target']
TARGET_COL = 'target'

X = df[FEATURE_COLS]
y = df[TARGET_COL]

# Split with stratification for classification
X_train, X_test, y_train, y_test = train_test_split(
    X, y,
    test_size=0.2,
    random_state=SEED,
    stratify=y  # For classification
)

print(f"Train: {X_train.shape}, Test: {X_test.shape}")
print(f"Train target distribution:\n{y_train.value_counts(normalize=True)}")
print(f"Test target distribution:\n{y_test.value_counts(normalize=True)}")

# Scale features (fit on train only!)
scaler = StandardScaler()
X_train_scaled = scaler.fit_transform(X_train)
X_test_scaled = scaler.transform(X_test)  # Transform only, no fit!
```

**Checklist**:
- [ ] Split before any train-dependent preprocessing
- [ ] Stratification used for imbalanced targets
- [ ] Scalers/encoders fit on train only
- [ ] Distributions verified similar

---

### 7. Model Definition

**Purpose**: Define model architecture and hyperparameters.

**Contents**:
- Model selection rationale
- Hyperparameter documentation
- Baseline model

**Template**:
```python
## 7. Model Definition

### 7.1 Baseline Model
from sklearn.dummy import DummyClassifier
baseline = DummyClassifier(strategy='most_frequent')
baseline.fit(X_train_scaled, y_train)
baseline_acc = baseline.score(X_test_scaled, y_test)
print(f"Baseline accuracy: {baseline_acc:.4f}")

### 7.2 Primary Model
from sklearn.ensemble import RandomForestClassifier

MODEL_PARAMS = {
    'n_estimators': 100,
    'max_depth': 10,
    'min_samples_split': 5,
    'random_state': SEED,
    'n_jobs': -1
}

model = RandomForestClassifier(**MODEL_PARAMS)
```

**Checklist**:
- [ ] Baseline established
- [ ] Hyperparameters documented
- [ ] Model choice justified
- [ ] Random state set

---

### 8. Training

**Purpose**: Fit model to training data.

**Contents**:
- Training code
- Training time logging
- Cross-validation (optional)

**Template**:
```python
## 8. Training

import time

### 8.1 Fit Model
start_time = time.time()
model.fit(X_train_scaled, y_train)
train_time = time.time() - start_time
print(f"Training time: {train_time:.2f} seconds")

### 8.2 Cross-Validation (optional)
from sklearn.model_selection import cross_val_score

cv_scores = cross_val_score(model, X_train_scaled, y_train, cv=5, scoring='accuracy')
print(f"CV Accuracy: {cv_scores.mean():.4f} (+/- {cv_scores.std()*2:.4f})")
```

**Checklist**:
- [ ] Training time logged
- [ ] No test data used
- [ ] CV scores reasonable

---

### 9. Evaluation

**Purpose**: Assess model performance on held-out test set.

**Contents**:
- Predictions
- Multiple metrics
- Visualizations (confusion matrix, ROC, etc.)

**Template**:
```python
## 9. Evaluation

### 9.1 Predictions
y_pred = model.predict(X_test_scaled)
y_prob = model.predict_proba(X_test_scaled)[:, 1]  # For binary

### 9.2 Metrics
from sklearn.metrics import (
    accuracy_score, precision_score, recall_score, f1_score,
    confusion_matrix, classification_report, roc_auc_score
)

print("Classification Report:")
print(classification_report(y_test, y_pred))

metrics = {
    'accuracy': accuracy_score(y_test, y_pred),
    'precision': precision_score(y_test, y_pred, average='weighted'),
    'recall': recall_score(y_test, y_pred, average='weighted'),
    'f1': f1_score(y_test, y_pred, average='weighted'),
    'roc_auc': roc_auc_score(y_test, y_prob)
}

for name, value in metrics.items():
    print(f"{name}: {value:.4f}")

### 9.3 Confusion Matrix
cm = confusion_matrix(y_test, y_pred)
plt.figure(figsize=(8, 6))
sns.heatmap(cm, annot=True, fmt='d', cmap='Blues')
plt.xlabel('Predicted')
plt.ylabel('Actual')
plt.title('Confusion Matrix')
plt.show()

### 9.4 ROC Curve
from sklearn.metrics import roc_curve

fpr, tpr, _ = roc_curve(y_test, y_prob)
plt.figure(figsize=(8, 6))
plt.plot(fpr, tpr, label=f'AUC = {metrics["roc_auc"]:.4f}')
plt.plot([0, 1], [0, 1], 'k--')
plt.xlabel('False Positive Rate')
plt.ylabel('True Positive Rate')
plt.title('ROC Curve')
plt.legend()
plt.show()
```

**Checklist**:
- [ ] Multiple metrics reported
- [ ] Confusion matrix visualized
- [ ] ROC curve (for classification)
- [ ] Comparison to baseline
- [ ] Results interpretable

---

### 10. Error Analysis

**Purpose**: Understand where and why the model fails.

**Contents**:
- Misclassified examples analysis
- Feature importance
- Performance by subgroup

**Template**:
```python
## 10. Error Analysis

### 10.1 Feature Importance
importances = pd.DataFrame({
    'feature': FEATURE_COLS,
    'importance': model.feature_importances_
}).sort_values('importance', ascending=False)

plt.figure(figsize=(10, 8))
sns.barplot(data=importances.head(20), x='importance', y='feature')
plt.title('Top 20 Feature Importances')
plt.show()

### 10.2 Misclassified Examples
test_df = X_test.copy()
test_df['y_true'] = y_test.values
test_df['y_pred'] = y_pred
test_df['correct'] = test_df['y_true'] == test_df['y_pred']

errors = test_df[~test_df['correct']]
print(f"Error count: {len(errors)} ({len(errors)/len(test_df)*100:.1f}%)")

### 10.3 Error Patterns
print("Errors by predicted class:")
print(errors['y_pred'].value_counts())
```

**Checklist**:
- [ ] Feature importances analyzed
- [ ] Misclassifications examined
- [ ] Patterns in errors identified
- [ ] Potential improvements noted

---

### 11. Save Artifacts

**Purpose**: Persist model and metadata for deployment.

**Contents**:
- Model serialization
- Scaler/encoder saving
- Metadata logging

**Template**:
```python
## 11. Save Artifacts

import joblib
from datetime import datetime

ARTIFACTS_DIR = Path('artifacts')
ARTIFACTS_DIR.mkdir(exist_ok=True)

timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')

### 11.1 Save Model
model_path = ARTIFACTS_DIR / f'model_{timestamp}.joblib'
joblib.dump(model, model_path)
print(f"Model saved to: {model_path}")

### 11.2 Save Scaler
scaler_path = ARTIFACTS_DIR / f'scaler_{timestamp}.joblib'
joblib.dump(scaler, scaler_path)
print(f"Scaler saved to: {scaler_path}")

### 11.3 Save Metadata
metadata = {
    'timestamp': timestamp,
    'model_params': MODEL_PARAMS,
    'feature_cols': FEATURE_COLS,
    'metrics': metrics,
    'train_size': len(X_train),
    'test_size': len(X_test),
    'seed': SEED
}

metadata_path = ARTIFACTS_DIR / f'metadata_{timestamp}.json'
with open(metadata_path, 'w') as f:
    json.dump(metadata, f, indent=2)
print(f"Metadata saved to: {metadata_path}")
```

**Checklist**:
- [ ] Model saved
- [ ] Preprocessors saved
- [ ] Metadata documented
- [ ] Versioned with timestamp

---

### 12. Conclusions

**Purpose**: Summarize findings and next steps.

**Contents**:
- Key results summary
- Limitations
- Recommendations
- Next steps

**Template**:
```markdown
## 12. Conclusions

### Key Results
- Model achieves X% accuracy vs Y% baseline
- Most important features: feature_a, feature_b, feature_c
- Model performs well on class A but struggles with class B

### Limitations
- Dataset limited to time period X-Y
- Feature Z not available in production
- Class imbalance may affect minority class predictions

### Recommendations
- Consider collecting more data for underrepresented classes
- Investigate feature_d which shows high importance
- A/B test against current production model

### Next Steps
1. Hyperparameter tuning with GridSearchCV
2. Try alternative models (XGBoost, LightGBM)
3. Feature engineering on date columns
4. Deploy to staging for integration testing
```

---

## Cell Organization Best Practices

### One Concept Per Cell
```python
# GOOD: Clear, focused cells
# Cell 1: Load data
df = pd.read_csv('data.csv')

# Cell 2: Check shape
print(df.shape)

# Cell 3: View sample
df.head()
```

```python
# BAD: Too much in one cell
df = pd.read_csv('data.csv')
print(df.shape)
df.head()
df.describe()
df.info()
# ... 50 more lines
```

### Markdown Headers for Navigation
Use hierarchical headers:
```markdown
## 3. Exploratory Data Analysis
### 3.1 Missing Values
### 3.2 Distributions
### 3.3 Correlations
```

### Output Cells Should Be Clean
```python
# GOOD: Formatted output
print(f"Accuracy: {accuracy:.4f}")
print(f"Precision: {precision:.4f}")

# BAD: Raw object dumps
model  # Just outputs repr
```

### Keep Cells Executable Independently
Each cell should run cleanly after kernel restart + run all above.
