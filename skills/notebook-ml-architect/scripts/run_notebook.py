#!/usr/bin/env python3
"""
Execute Jupyter notebook with parameters.

Uses papermill for parameterized execution, falls back to nbclient if papermill
is not available.

Usage:
    python run_notebook.py input.ipynb output.ipynb [--params '{"key": "value"}'] [--timeout 3600]

Features:
    - Parameterized execution (papermill-style parameters cell)
    - Configurable timeout
    - Error capture and reporting
    - Kernel specification override
"""

import argparse
import json
import sys
from pathlib import Path

# Try papermill first, fall back to nbclient
try:
    import papermill as pm

    HAS_PAPERMILL = True
except ImportError:
    HAS_PAPERMILL = False

try:
    import nbformat
    from nbclient import NotebookClient
    from nbclient.exceptions import CellExecutionError

    HAS_NBCLIENT = True
except ImportError:
    HAS_NBCLIENT = False

if not HAS_PAPERMILL and not HAS_NBCLIENT:
    print("Error: Neither papermill nor nbclient installed.")
    print("Install one of: pip install papermill OR pip install nbclient nbformat")
    sys.exit(1)


def run_with_papermill(
    input_path: str,
    output_path: str,
    parameters: dict | None = None,
    timeout: int = 600,
    kernel_name: str | None = None,
) -> dict:
    """Execute notebook using papermill."""
    kwargs = {
        "input_path": input_path,
        "output_path": output_path,
        "parameters": parameters or {},
        "kernel_name": kernel_name,
        "progress_bar": True,
    }

    # Remove None values
    kwargs = {k: v for k, v in kwargs.items() if v is not None}

    try:
        nb = pm.execute_notebook(**kwargs)
        return {
            "success": True,
            "output_path": output_path,
            "cells_executed": len(nb.cells),
        }
    except pm.PapermillExecutionError as e:
        return {
            "success": False,
            "error": str(e),
            "output_path": output_path,
            "cell_index": getattr(e, "cell_index", None),
        }


def run_with_nbclient(
    input_path: str,
    output_path: str,
    parameters: dict | None = None,
    timeout: int = 600,
    kernel_name: str | None = None,
) -> dict:
    """Execute notebook using nbclient (fallback)."""
    # Read notebook
    nb = nbformat.read(input_path, as_version=4)

    # Inject parameters if provided
    if parameters:
        # Find or create parameters cell
        param_cell_idx = None
        for idx, cell in enumerate(nb.cells):
            if cell.cell_type == "code" and "parameters" in cell.metadata.get(
                "tags", []
            ):
                param_cell_idx = idx
                break

        # Create parameter assignments
        param_code = "# Parameters (injected)\n"
        for key, value in parameters.items():
            param_code += f"{key} = {json.dumps(value)}\n"

        if param_cell_idx is not None:
            # Append to existing parameters cell
            nb.cells[param_cell_idx].source += "\n" + param_code
        else:
            # Insert new cell at position 1 (after imports typically)
            new_cell = nbformat.v4.new_code_cell(source=param_code)
            new_cell.metadata["tags"] = ["injected-parameters"]
            nb.cells.insert(1, new_cell)

    # Configure client
    client_kwargs = {
        "nb": nb,
        "timeout": timeout,
        "kernel_name": kernel_name or nb.metadata.get("kernelspec", {}).get("name"),
    }
    client_kwargs = {k: v for k, v in client_kwargs.items() if v is not None}

    client = NotebookClient(**client_kwargs)

    try:
        # Execute
        client.execute()

        # Save output
        nbformat.write(nb, output_path)

        return {
            "success": True,
            "output_path": output_path,
            "cells_executed": len(nb.cells),
        }

    except CellExecutionError as e:
        # Save even on error (preserves partial outputs)
        nbformat.write(nb, output_path)

        return {
            "success": False,
            "error": str(e),
            "output_path": output_path,
            "cell_index": getattr(e, "cell_index", None),
        }


def run_notebook(
    input_path: str,
    output_path: str,
    parameters: dict | None = None,
    timeout: int = 600,
    kernel_name: str | None = None,
) -> dict:
    """Execute notebook with the best available backend."""
    # Validate paths
    input_p = Path(input_path)
    if not input_p.exists():
        return {"success": False, "error": f"Input notebook not found: {input_path}"}

    output_p = Path(output_path)
    output_p.parent.mkdir(parents=True, exist_ok=True)

    print(f"Executing: {input_path}")
    print(f"Output: {output_path}")
    if parameters:
        print(f"Parameters: {json.dumps(parameters, indent=2)}")
    print(f"Timeout: {timeout}s")
    print(f"Backend: {'papermill' if HAS_PAPERMILL else 'nbclient'}")
    print("-" * 40)

    if HAS_PAPERMILL:
        result = run_with_papermill(
            input_path, output_path, parameters, timeout, kernel_name
        )
    else:
        result = run_with_nbclient(
            input_path, output_path, parameters, timeout, kernel_name
        )

    return result


def main():
    parser = argparse.ArgumentParser(
        description="Execute Jupyter notebook with parameters"
    )
    parser.add_argument("input", help="Input notebook path")
    parser.add_argument("output", help="Output notebook path")
    parser.add_argument(
        "--params",
        "-p",
        type=str,
        default="{}",
        help='Parameters as JSON string: \'{"key": "value"}\'',
    )
    parser.add_argument(
        "--timeout",
        "-t",
        type=int,
        default=600,
        help="Execution timeout in seconds (default: 600)",
    )
    parser.add_argument(
        "--kernel", "-k", type=str, help="Kernel name to use (default: from notebook)"
    )

    args = parser.parse_args()

    # Parse parameters
    try:
        parameters = json.loads(args.params)
    except json.JSONDecodeError as e:
        print(f"Error parsing parameters JSON: {e}", file=sys.stderr)
        sys.exit(1)

    # Run notebook
    result = run_notebook(
        input_path=args.input,
        output_path=args.output,
        parameters=parameters,
        timeout=args.timeout,
        kernel_name=args.kernel,
    )

    # Report result
    print("-" * 40)
    if result["success"]:
        print("SUCCESS: Notebook executed successfully")
        print(f"Output saved to: {result['output_path']}")
        if "cells_executed" in result:
            print(f"Cells executed: {result['cells_executed']}")
    else:
        print(f"FAILED: {result.get('error', 'Unknown error')}")
        if result.get("cell_index") is not None:
            print(f"Failed at cell: {result['cell_index']}")
        print(f"Partial output saved to: {result.get('output_path', 'N/A')}")
        sys.exit(1)


if __name__ == "__main__":
    main()
