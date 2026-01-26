"""
Evaluation Block for ML Notebooks

Comprehensive evaluation utilities for classification and regression tasks.
Copy relevant functions to your notebook's evaluation section.

Contents:
    - evaluate_classifier: Full classification metrics and visualizations
    - evaluate_regressor: Full regression metrics and visualizations
    - plot_learning_curves: Training/validation curves
    - plot_feature_importance: Feature importance visualization
"""

from typing import Any

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd


# =============================================================================
# Classification Evaluation
# =============================================================================


def evaluate_classifier(
    y_true: np.ndarray,
    y_pred: np.ndarray,
    y_prob: np.ndarray | None = None,
    class_names: list[str] | None = None,
    figsize: tuple[int, int] = (12, 5),
) -> dict[str, float]:
    """
    Comprehensive classification evaluation with metrics and visualizations.

    Args:
        y_true: True labels
        y_pred: Predicted labels
        y_prob: Predicted probabilities (for ROC/AUC)
        class_names: Names for each class
        figsize: Figure size for plots

    Returns:
        Dictionary of computed metrics
    """
    from sklearn.metrics import (
        accuracy_score,
        classification_report,
        confusion_matrix,
        f1_score,
        precision_score,
        recall_score,
        roc_auc_score,
        roc_curve,
    )

    # Compute metrics
    metrics = {
        "accuracy": accuracy_score(y_true, y_pred),
        "precision_weighted": precision_score(
            y_true, y_pred, average="weighted", zero_division=0
        ),
        "recall_weighted": recall_score(
            y_true, y_pred, average="weighted", zero_division=0
        ),
        "f1_weighted": f1_score(y_true, y_pred, average="weighted", zero_division=0),
        "precision_macro": precision_score(
            y_true, y_pred, average="macro", zero_division=0
        ),
        "recall_macro": recall_score(y_true, y_pred, average="macro", zero_division=0),
        "f1_macro": f1_score(y_true, y_pred, average="macro", zero_division=0),
    }

    # ROC AUC for binary classification
    if y_prob is not None:
        try:
            if len(np.unique(y_true)) == 2:
                # Binary classification
                if y_prob.ndim == 2:
                    y_prob_pos = y_prob[:, 1]
                else:
                    y_prob_pos = y_prob
                metrics["roc_auc"] = roc_auc_score(y_true, y_prob_pos)
            else:
                # Multi-class
                metrics["roc_auc_ovr"] = roc_auc_score(
                    y_true, y_prob, multi_class="ovr"
                )
        except ValueError:
            pass

    # Print classification report
    print("Classification Report:")
    print("=" * 60)
    print(
        classification_report(y_true, y_pred, target_names=class_names, zero_division=0)
    )

    # Print summary metrics
    print("\nSummary Metrics:")
    print("-" * 40)
    for name, value in metrics.items():
        print(f"  {name}: {value:.4f}")

    # Create visualizations
    fig, axes = plt.subplots(
        1,
        2 if y_prob is not None and len(np.unique(y_true)) == 2 else 1,
        figsize=figsize,
    )

    if not isinstance(axes, np.ndarray):
        axes = [axes]

    # Confusion Matrix
    import seaborn as sns

    cm = confusion_matrix(y_true, y_pred)
    sns.heatmap(
        cm,
        annot=True,
        fmt="d",
        cmap="Blues",
        xticklabels=class_names,
        yticklabels=class_names,
        ax=axes[0],
    )
    axes[0].set_xlabel("Predicted")
    axes[0].set_ylabel("Actual")
    axes[0].set_title("Confusion Matrix")

    # ROC Curve (binary only)
    if y_prob is not None and len(np.unique(y_true)) == 2 and len(axes) > 1:
        if y_prob.ndim == 2:
            y_prob_pos = y_prob[:, 1]
        else:
            y_prob_pos = y_prob

        fpr, tpr, _ = roc_curve(y_true, y_prob_pos)
        axes[1].plot(fpr, tpr, label=f"AUC = {metrics.get('roc_auc', 0):.4f}")
        axes[1].plot([0, 1], [0, 1], "k--", label="Random")
        axes[1].set_xlabel("False Positive Rate")
        axes[1].set_ylabel("True Positive Rate")
        axes[1].set_title("ROC Curve")
        axes[1].legend()

    plt.tight_layout()
    plt.show()

    return metrics


# =============================================================================
# Regression Evaluation
# =============================================================================


def evaluate_regressor(
    y_true: np.ndarray,
    y_pred: np.ndarray,
    figsize: tuple[int, int] = (12, 5),
) -> dict[str, float]:
    """
    Comprehensive regression evaluation with metrics and visualizations.

    Args:
        y_true: True values
        y_pred: Predicted values
        figsize: Figure size for plots

    Returns:
        Dictionary of computed metrics
    """
    from sklearn.metrics import (
        mean_absolute_error,
        mean_absolute_percentage_error,
        mean_squared_error,
        r2_score,
    )

    # Compute metrics
    metrics = {
        "r2": r2_score(y_true, y_pred),
        "mae": mean_absolute_error(y_true, y_pred),
        "mse": mean_squared_error(y_true, y_pred),
        "rmse": np.sqrt(mean_squared_error(y_true, y_pred)),
        "mape": mean_absolute_percentage_error(y_true, y_pred) * 100,
    }

    # Print summary
    print("Regression Metrics:")
    print("=" * 60)
    for name, value in metrics.items():
        print(f"  {name}: {value:.4f}")

    # Create visualizations
    fig, axes = plt.subplots(1, 2, figsize=figsize)

    # Actual vs Predicted
    axes[0].scatter(y_true, y_pred, alpha=0.5)
    min_val = min(y_true.min(), y_pred.min())
    max_val = max(y_true.max(), y_pred.max())
    axes[0].plot([min_val, max_val], [min_val, max_val], "r--", label="Perfect")
    axes[0].set_xlabel("Actual")
    axes[0].set_ylabel("Predicted")
    axes[0].set_title(f"Actual vs Predicted (R² = {metrics['r2']:.4f})")
    axes[0].legend()

    # Residuals
    residuals = y_true - y_pred
    axes[1].hist(residuals, bins=30, edgecolor="black", alpha=0.7)
    axes[1].axvline(x=0, color="r", linestyle="--")
    axes[1].set_xlabel("Residual")
    axes[1].set_ylabel("Frequency")
    axes[1].set_title(f"Residual Distribution (MAE = {metrics['mae']:.4f})")

    plt.tight_layout()
    plt.show()

    return metrics


# =============================================================================
# Learning Curves
# =============================================================================


def plot_learning_curves(
    train_scores: list[float],
    val_scores: list[float],
    train_losses: list[float] | None = None,
    val_losses: list[float] | None = None,
    metric_name: str = "Accuracy",
    figsize: tuple[int, int] = (12, 5),
) -> None:
    """
    Plot training and validation learning curves.

    Args:
        train_scores: Training scores per epoch
        val_scores: Validation scores per epoch
        train_losses: Training losses per epoch (optional)
        val_losses: Validation losses per epoch (optional)
        metric_name: Name of the metric being plotted
        figsize: Figure size
    """
    epochs = range(1, len(train_scores) + 1)

    if train_losses is not None and val_losses is not None:
        fig, axes = plt.subplots(1, 2, figsize=figsize)

        # Scores
        axes[0].plot(epochs, train_scores, "b-", label="Training")
        axes[0].plot(epochs, val_scores, "r-", label="Validation")
        axes[0].set_xlabel("Epoch")
        axes[0].set_ylabel(metric_name)
        axes[0].set_title(f"{metric_name} vs Epoch")
        axes[0].legend()
        axes[0].grid(True, alpha=0.3)

        # Losses
        axes[1].plot(epochs, train_losses, "b-", label="Training")
        axes[1].plot(epochs, val_losses, "r-", label="Validation")
        axes[1].set_xlabel("Epoch")
        axes[1].set_ylabel("Loss")
        axes[1].set_title("Loss vs Epoch")
        axes[1].legend()
        axes[1].grid(True, alpha=0.3)

    else:
        fig, ax = plt.subplots(figsize=(figsize[0] // 2, figsize[1]))
        ax.plot(epochs, train_scores, "b-", label="Training")
        ax.plot(epochs, val_scores, "r-", label="Validation")
        ax.set_xlabel("Epoch")
        ax.set_ylabel(metric_name)
        ax.set_title(f"{metric_name} vs Epoch")
        ax.legend()
        ax.grid(True, alpha=0.3)

    plt.tight_layout()
    plt.show()


# =============================================================================
# Feature Importance
# =============================================================================


def plot_feature_importance(
    feature_names: list[str],
    importances: np.ndarray,
    top_n: int = 20,
    figsize: tuple[int, int] = (10, 8),
) -> pd.DataFrame:
    """
    Plot feature importance.

    Args:
        feature_names: List of feature names
        importances: Array of importance values
        top_n: Number of top features to show
        figsize: Figure size

    Returns:
        DataFrame with feature importances
    """
    import seaborn as sns

    # Create DataFrame
    importance_df = pd.DataFrame(
        {"feature": feature_names, "importance": importances}
    ).sort_values("importance", ascending=False)

    # Plot
    plt.figure(figsize=figsize)
    sns.barplot(
        data=importance_df.head(top_n),
        x="importance",
        y="feature",
        palette="viridis",
    )
    plt.xlabel("Importance")
    plt.ylabel("Feature")
    plt.title(f"Top {top_n} Feature Importances")
    plt.tight_layout()
    plt.show()

    return importance_df


# =============================================================================
# Cross-Validation Summary
# =============================================================================


def print_cv_summary(cv_results: dict[str, Any]) -> pd.DataFrame:
    """
    Print cross-validation results summary.

    Args:
        cv_results: Results from cross_validate() or similar

    Returns:
        Summary DataFrame
    """
    summary = []
    for key, values in cv_results.items():
        if key.startswith("test_") or key.startswith("train_"):
            summary.append(
                {
                    "metric": key,
                    "mean": np.mean(values),
                    "std": np.std(values),
                    "min": np.min(values),
                    "max": np.max(values),
                }
            )

    df = pd.DataFrame(summary)
    print("Cross-Validation Results:")
    print("=" * 60)
    print(df.to_string(index=False))
    return df
