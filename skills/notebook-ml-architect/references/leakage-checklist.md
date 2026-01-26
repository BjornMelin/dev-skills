# Data Leakage Checklist

Comprehensive taxonomy of data leakage types in ML pipelines, based on Princeton research and ML best practices. Use this checklist to audit notebooks for leakage issues.

## What is Data Leakage?

Data leakage occurs when information from outside the training dataset is used to create the model, leading to overly optimistic performance estimates that don't generalize to real-world data.

**Impact**: Models appear to perform well in notebooks but fail in production.

---

## Leakage Type 1: No Train/Test Separation

### Description
Training and test data are not separated during preprocessing, modeling, or evaluation steps. The model has seen test data before evaluation.

### Detection Patterns

```python
# BAD: Fit on all data
scaler.fit(X)  # Sees all data including test
X_train, X_test = train_test_split(X)

# BAD: Feature engineering on all data
df['feature'] = compute_feature(df)  # Uses all rows
df_train, df_test = split(df)

# BAD: Imputation on all data
df.fillna(df.mean())  # Mean includes test data
```

### Correct Pattern

```python
# GOOD: Split first, fit on train only
X_train, X_test = train_test_split(X)
scaler.fit(X_train)
X_train = scaler.transform(X_train)
X_test = scaler.transform(X_test)
```

### Checklist
- [ ] `train_test_split()` called before any preprocessing
- [ ] All `.fit()` calls use only training data
- [ ] All `.transform()` uses fitted parameters from train
- [ ] Feature engineering uses only training data statistics

---

## Leakage Type 2: Feature Selection on Train+Test

### Description
Feature selection (removing low-variance features, selecting top-k, etc.) is performed on the combined dataset before splitting.

### Detection Patterns

```python
# BAD: Select features using all data
selector = SelectKBest(k=10)
X_selected = selector.fit_transform(X, y)  # Sees all data
X_train, X_test = train_test_split(X_selected)

# BAD: Remove low-variance features on all data
low_var = X.var() < 0.01
X = X.loc[:, ~low_var]
```

### Correct Pattern

```python
# GOOD: Feature selection on train only
X_train, X_test = train_test_split(X, y)
selector = SelectKBest(k=10)
X_train = selector.fit_transform(X_train, y_train)
X_test = selector.transform(X_test)
```

### Checklist
- [ ] Feature selection after train/test split
- [ ] Feature importance computed on training data only
- [ ] Same features applied to test (no re-selection)

---

## Leakage Type 3: Preprocessing on Train+Test Together

### Description
Preprocessing operations (scaling, normalization, encoding) are fitted on the full dataset.

### Detection Patterns

```python
# BAD: Scale using full dataset statistics
X_scaled = (X - X.mean()) / X.std()

# BAD: Label encoding using all categories
encoder = LabelEncoder()
df['category'] = encoder.fit_transform(df['category'])

# BAD: Target encoding using all data
df['encoded'] = df.groupby('category')['target'].transform('mean')
```

### Correct Pattern

```python
# GOOD: Scale using training statistics only
X_train, X_test = train_test_split(X)
mean, std = X_train.mean(), X_train.std()
X_train_scaled = (X_train - mean) / std
X_test_scaled = (X_test - mean) / std
```

### Checklist
- [ ] StandardScaler/MinMaxScaler fit on train only
- [ ] Encoders fit on training categories only
- [ ] Target encoding uses only training data statistics
- [ ] PCA/dimensionality reduction fit on train only

---

## Leakage Type 4: Non-Independence (Duplicates/Overlap)

### Description
The same or highly similar samples appear in both training and test sets, or there are dependencies between samples.

### Detection Patterns

```python
# BAD: Duplicates in data
df = pd.concat([df1, df2])  # May have overlapping rows
X_train, X_test = train_test_split(df)

# BAD: Time series without proper splitting
# Random split on time-ordered data
df_shuffled = df.sample(frac=1)
X_train, X_test = train_test_split(df_shuffled)

# BAD: Group leakage (same patient in train and test)
# Medical images from same patient in both sets
```

### Correct Pattern

```python
# GOOD: Remove duplicates first
df = df.drop_duplicates()

# GOOD: Time-based split for time series
train = df[df['date'] < cutoff_date]
test = df[df['date'] >= cutoff_date]

# GOOD: Group-aware split
from sklearn.model_selection import GroupShuffleSplit
gss = GroupShuffleSplit(n_splits=1, test_size=0.2)
train_idx, test_idx = next(gss.split(X, y, groups=patient_ids))
```

### Checklist
- [ ] Check for duplicate rows before splitting
- [ ] Use time-based split for temporal data
- [ ] Use GroupKFold/GroupShuffleSplit for grouped data
- [ ] Verify no sample appears in both train and test

---

## Leakage Type 5: Temporal Leakage

### Description
Using future information to predict past events. Common in time series and event prediction.

### Detection Patterns

```python
# BAD: Feature uses future data
df['next_day_price'] = df['price'].shift(-1)
# Using tomorrow's price to predict today

# BAD: Random split on time series
train, test = train_test_split(time_series_df)

# BAD: Lag features computed on full series
df['rolling_mean'] = df['value'].rolling(7).mean()
# Rolling mean at time t uses values up to t, but computed on full series
```

### Correct Pattern

```python
# GOOD: Only use past data
df['prev_day_price'] = df['price'].shift(1)

# GOOD: Time-based split
train = df[df['date'] < '2024-01-01']
test = df[df['date'] >= '2024-01-01']

# GOOD: Compute rolling features separately
train['rolling_mean'] = train['value'].rolling(7).mean()
```

### Checklist
- [ ] All features computed from past data only
- [ ] Time-based train/test split (no random shuffle)
- [ ] No future-looking aggregations
- [ ] Event prediction uses only pre-event features

---

## Leakage Type 6: Illegitimate Features (Proxies)

### Description
Features that are proxies for the target variable or contain information that wouldn't be available at prediction time.

### Detection Patterns

```python
# BAD: ID that correlates with target
# Hospital ID encodes patient outcome (some hospitals have higher mortality)
model.fit(df[['hospital_id', 'age', 'symptoms']], df['outcome'])

# BAD: Feature derived from target
df['will_churn'] = df['last_activity_date'] > cutoff
# 'will_churn' directly encodes the target

# BAD: Post-hoc feature
df['treatment_outcome'] = ...  # Known only after treatment
model.fit(df, df['responded_to_treatment'])
```

### Correct Pattern

```python
# GOOD: Only use features available at prediction time
features = ['age', 'symptoms', 'prior_visits']
# Exclude hospital_id or use proper encoding

# GOOD: Clearly define prediction point
# Only use features known BEFORE the event you're predicting
```

### Checklist
- [ ] All features available at prediction time
- [ ] No features derived from target
- [ ] No post-hoc features (known only after outcome)
- [ ] ID columns checked for spurious correlations

---

## Leakage Type 7: Sampling Bias

### Description
Training and test sets are not representative of the true data distribution due to biased sampling.

### Detection Patterns

```python
# BAD: Class imbalance handled before split
# Oversampling on full dataset
from imblearn.over_sampling import SMOTE
X_resampled, y_resampled = SMOTE().fit_resample(X, y)
X_train, X_test = train_test_split(X_resampled, y_resampled)
# Test set now has synthetic samples!

# BAD: Stratified split on derived feature
# Stratifying on a feature that won't exist in production
```

### Correct Pattern

```python
# GOOD: Oversample training data only
X_train, X_test, y_train, y_test = train_test_split(X, y, stratify=y)
X_train_resampled, y_train_resampled = SMOTE().fit_resample(X_train, y_train)
# Test set remains untouched
```

### Checklist
- [ ] Oversampling/undersampling on train only
- [ ] Stratified split uses original labels
- [ ] Test set reflects production distribution
- [ ] No synthetic samples in test set

---

## Leakage Type 8: Test Set Not From Distribution of Interest

### Description
The test set doesn't match the distribution where the model will be deployed.

### Detection Patterns

```python
# BAD: Test on same time period as train
# Both train and test from 2023, but deploying in 2024

# BAD: Test on same geographic region
# Train and test from US, but deploying globally

# BAD: Test on same data source
# Train and test from Hospital A, deploying to Hospital B
```

### Correct Pattern

```python
# GOOD: Test on future data
train = df[df['year'] < 2024]
test = df[df['year'] == 2024]

# GOOD: Test on held-out domain
train = df[df['region'] != 'EU']
test = df[df['region'] == 'EU']
```

### Checklist
- [ ] Test set represents deployment scenario
- [ ] Temporal validation for time-sensitive models
- [ ] Geographic/domain holdout for transfer scenarios
- [ ] Test set from realistic data collection process

---

## Quick Audit Checklist

Run through this for every notebook:

1. **Split Point**
   - [ ] Where is `train_test_split` called?
   - [ ] Is it before ALL preprocessing?

2. **Fit/Transform Pattern**
   - [ ] Search for `.fit(` and `.fit_transform(`
   - [ ] Are they all on training data only?

3. **Data Dependencies**
   - [ ] Are there duplicates?
   - [ ] Are there group dependencies?
   - [ ] Is this time series data?

4. **Feature Engineering**
   - [ ] When are features computed?
   - [ ] Do any features use target information?
   - [ ] Are features available at prediction time?

5. **Sampling**
   - [ ] Is oversampling done after split?
   - [ ] Does test set reflect production?

## References

- [Leakage and the Reproducibility Crisis in ML-based Science](https://reproducible.cs.princeton.edu/)
- [Kaufman et al. - Leakage in Data Mining](https://dl.acm.org/doi/10.1145/2382577.2382579)
- [sklearn User Guide - Common Pitfalls](https://scikit-learn.org/stable/common_pitfalls.html)
