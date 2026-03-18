# Feature Document Template

**Version**: X.Y.Z  
**Feature**: Feature Name  
**Status**: [Planned | In Progress | Completed]  
**Created**: YYYY-MM-DD  
**Completed**: YYYY-MM-DD (if completed)

---

## Overview

Brief description of what this feature does and why it matters.

---

## Based On

- [ARCHITECTURE.md](ARCHITECTURE.md) - Relevant sections
- [FEATURES.md](FEATURES.md) - Feature specification
- [IMPLEMENTATION.md](IMPLEMENTATION.md) - Implementation phase

---

## Scope

### In Scope

- Item 1
- Item 2

### Out of Scope

- Item 1
- Item 2

---

## Design

### Architecture

Describe how this fits into the overall system.

```
┌─────────────────────────────────────┐
│           Component                  │
│  ┌─────────────┐  ┌─────────────┐   │
│  │   Part A   │  │   Part B   │   │
│  └─────────────┘  └─────────────┘   │
└─────────────────────────────────────┘
```

### Data Structures

```rust
// Key structs, types, etc.
```

### APIs

```rust
// Key functions, traits, etc.
```

---

## Implementation Steps

### Step 1: Name

What to do and why.

```rust
// Code example
```

### Step 2: Name

...

---

## File Structure

```
src/
  ├── module_a.rs
  └── module_b.rs
```

---

## Testing

### Manual Tests

| Test | Input | Expected Output |
|------|-------|----------------|
| Test 1 | X | Y |

### Automated Tests

- [ ] Unit tests for X
- [ ] Integration tests for Y

---

## Success Criteria

- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

---

## Notes

Any additional notes, considerations, or open questions.

---

## Changelog

| Date | Change |
|------|--------|
| YYYY-MM-DD | Initial version |
