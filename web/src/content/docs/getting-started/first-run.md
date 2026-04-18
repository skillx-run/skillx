---
title: First Run
description: The shortest path to a successful skillx run with a real GitHub skill.
---

## 1. Install skillx

If you have not installed skillx yet, use the recommended install command:

```bash
curl -fsSL https://skillx.run/install.sh | sh
```

## 2. Run a Real GitHub Skill

Use one of the official examples from GitHub:

```bash
skillx run github:skillx-run/skillx/examples/skills/name-poem "Your Name"
```

## 3. Expect This Kind of Output

You should see a flow like this:

```text
● Resolving source...
✓ Resolved: name-poem
● Scanning for security issues...
✓ PASS - no findings
● Injecting skill...
✓ Injected 1 files
● Launching agent...
```

## 4. What skillx Is Doing

skillx fetches the skill from GitHub, scans it before any injection happens, copies the skill into the right agent-specific location, launches your agent with the skill in context, and then cleans up the temporary files when the run finishes.

That sequence is the point of the tool: one command to run a skill safely without leaving a permanent install behind.

## Next Steps

- [Famous Skills](/getting-started/famous-skills/) - try the most useful skills next
- [Official Examples](/examples/overview/) - browse the full set of runnable examples
- [Run Skills](/cli/run/) - learn the command in detail
