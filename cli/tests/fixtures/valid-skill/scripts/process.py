#!/usr/bin/env python3
"""PDF processing script — safe, no external calls."""

import sys
import os


def extract_text(filepath):
    """Extract text from a PDF file."""
    with open(filepath, 'r') as f:
        return f.read()


def main():
    if len(sys.argv) < 2:
        print("Usage: process.py <file>")
        sys.exit(1)

    filepath = sys.argv[1]
    if not os.path.exists(filepath):
        print(f"File not found: {filepath}")
        sys.exit(1)

    text = extract_text(filepath)
    print(text)


if __name__ == "__main__":
    main()
