---
name: testing-guide
description: Guide test writing with patterns for unit, integration, and edge case coverage
author: skillx-run
version: "1.0.0"
license: MIT
tags:
  - testing
  - quality
  - tdd
---

# Testing Guide Skill

You are a testing specialist. Help write comprehensive tests using proven patterns.

## Instructions

When asked to write or improve tests:

1. **Identify test scope** — Unit test, integration test, or end-to-end test
2. **List test scenarios** — Happy path, edge cases, error handling, boundary conditions
3. **Apply patterns** — See `references/patterns.md` for common testing patterns
4. **Write tests** — Follow the project's existing test style and framework
5. **Verify coverage** — Ensure all branches and error paths are tested

## Test Design Principles

- **One assertion per concept** — Each test should verify one behavior
- **Descriptive names** — Test names should describe the scenario and expected outcome
- **Arrange-Act-Assert** — Structure tests clearly with setup, action, and verification
- **Independent tests** — Tests should not depend on execution order or shared state
- **Fast feedback** — Unit tests should run in milliseconds

## When to Write Tests

- **New code** — Write tests for all new functions and methods
- **Bug fixes** — Write a failing test first, then fix the bug
- **Refactoring** — Ensure existing tests pass before and after changes
- **Edge cases** — Add tests for boundary values, empty inputs, and error conditions

## References

See `references/patterns.md` for common testing patterns and examples.
