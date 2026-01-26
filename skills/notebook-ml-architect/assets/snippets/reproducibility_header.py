"""
Reproducibility Header for ML Notebooks

Copy this code to the first code cell of your notebook to ensure reproducible results.
Handles seeding for: random, numpy, torch, tensorflow.

Usage:
    1. Copy this entire file content to your first code cell
    2. Adjust SEED value as needed
    3. Add any additional library version prints
"""

import os
import random
import sys
from datetime import datetime

import numpy as np

# =============================================================================
# Configuration
# =============================================================================

SEED = 42
PROJECT_ROOT = os.path.dirname(os.path.abspath("__file__"))


# =============================================================================
# Random Seed Setting
# =============================================================================


def set_seeds(seed: int = SEED) -> None:
    """
    Set random seeds for reproducibility across all common ML libraries.

    Args:
        seed: Random seed value (default: 42)
    """
    # Python's random module
    random.seed(seed)

    # NumPy
    np.random.seed(seed)

    # Python hash seed (for dict ordering in Python 3.7+)
    os.environ["PYTHONHASHSEED"] = str(seed)

    # PyTorch (if available)
    try:
        import torch

        torch.manual_seed(seed)
        torch.cuda.manual_seed(seed)
        torch.cuda.manual_seed_all(seed)  # For multi-GPU

        # Deterministic operations (may impact performance)
        torch.backends.cudnn.deterministic = True
        torch.backends.cudnn.benchmark = False

        # For PyTorch 1.8+
        if hasattr(torch, "use_deterministic_algorithms"):
            try:
                torch.use_deterministic_algorithms(True)
            except Exception:
                pass  # Some ops don't have deterministic implementations
    except ImportError:
        pass

    # TensorFlow (if available)
    try:
        import tensorflow as tf

        tf.random.set_seed(seed)

        # Disable GPU non-determinism
        os.environ["TF_DETERMINISTIC_OPS"] = "1"
        os.environ["TF_CUDNN_DETERMINISTIC"] = "1"
    except ImportError:
        pass

    print(f"Random seeds set to: {seed}")


# =============================================================================
# Environment Capture
# =============================================================================


def print_environment() -> dict:
    """
    Print and return environment information for reproducibility.

    Returns:
        Dictionary with environment details
    """
    env_info = {
        "timestamp": datetime.now().isoformat(),
        "python_version": sys.version,
        "platform": sys.platform,
        "seed": SEED,
        "packages": {},
    }

    print("=" * 60)
    print("Environment Information")
    print("=" * 60)
    print(f"Timestamp: {env_info['timestamp']}")
    print(f"Python: {env_info['python_version']}")
    print(f"Platform: {env_info['platform']}")
    print(f"Seed: {env_info['seed']}")
    print()

    # Core packages
    packages = [
        "numpy",
        "pandas",
        "sklearn",
        "scipy",
        "matplotlib",
        "seaborn",
        "torch",
        "tensorflow",
        "keras",
        "xgboost",
        "lightgbm",
        "catboost",
    ]

    print("Package Versions:")
    for pkg_name in packages:
        try:
            pkg = __import__(pkg_name)
            version = getattr(pkg, "__version__", "unknown")
            env_info["packages"][pkg_name] = version
            print(f"  {pkg_name}: {version}")
        except ImportError:
            pass

    print("=" * 60)
    return env_info


# =============================================================================
# Initialize
# =============================================================================

# Set seeds immediately
set_seeds(SEED)

# Print environment info
ENV_INFO = print_environment()


# =============================================================================
# Optional: GPU Information
# =============================================================================


def print_gpu_info() -> None:
    """Print GPU information if available."""
    print("\nGPU Information:")

    # PyTorch GPU
    try:
        import torch

        if torch.cuda.is_available():
            print(f"  PyTorch CUDA: {torch.version.cuda}")
            print(f"  GPU Count: {torch.cuda.device_count()}")
            for i in range(torch.cuda.device_count()):
                print(f"  GPU {i}: {torch.cuda.get_device_name(i)}")
        else:
            print("  PyTorch: No CUDA available")
    except ImportError:
        pass

    # TensorFlow GPU
    try:
        import tensorflow as tf

        gpus = tf.config.list_physical_devices("GPU")
        if gpus:
            print(f"  TensorFlow GPUs: {len(gpus)}")
            for gpu in gpus:
                print(f"    {gpu.name}")
        else:
            print("  TensorFlow: No GPUs available")
    except ImportError:
        pass


# Uncomment to print GPU info:
# print_gpu_info()
