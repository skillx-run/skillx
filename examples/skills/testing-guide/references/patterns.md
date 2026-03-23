# Common Testing Patterns

## 1. Boundary Value Analysis

Test at the edges of valid input ranges:

- Minimum valid value
- Maximum valid value
- Just below minimum (invalid)
- Just above maximum (invalid)
- Zero / empty / null

## 2. Equivalence Partitioning

Group inputs into classes that should produce the same behavior:

- Valid inputs (one test per class)
- Invalid inputs (one test per class)
- Special values (empty string, negative numbers, Unicode)

## 3. Error Path Testing

Ensure every error path is exercised:

- Network failures (timeout, connection refused)
- File system errors (not found, permission denied)
- Invalid input (malformed data, wrong types)
- Resource exhaustion (out of memory, disk full)

## 4. State Transition Testing

For stateful systems, test:

- Valid state transitions
- Invalid state transitions (should be rejected)
- Concurrent state changes
- Recovery from error states

## 5. Test Doubles

Choose the right type of test double:

- **Stub** — Returns predetermined responses
- **Mock** — Verifies interactions (calls, arguments)
- **Fake** — Working implementation (in-memory database)
- **Spy** — Records calls for later verification
