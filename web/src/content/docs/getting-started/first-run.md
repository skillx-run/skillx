---
title: First Run
description: The shortest path to a successful skillx run with a real Agent Skill.
---

## 1. Install skillx

If you have not installed skillx yet, use the recommended install command:

```bash
curl -fsSL https://skillx.run/install.sh | sh
```

## 2. Run a Real Agent Skill

Use the same public Agent Skill featured on the landing page:

```bash
skillx run github:anthropics/skills/skills/frontend-design "Redesign the hero section."
```

## 3. Expect This Kind of Output

You should see a flow like this:

```text
● Resolving source...
✓ Resolved: frontend-design
● Scanning for security issues...
✓ PASS - no findings
● Injecting skill...
✓ Injected 1 files
● Launching agent...
```

## 4. What skillx Is Doing

skillx fetches the skill from GitHub, scans it before any injection happens, copies the skill into the right agent-specific location, launches your agent with the skill in context, and then cleans up the temporary files when the run finishes.

This is a stronger first run than a toy example: you get a visible result from a real upstream skill without committing to a permanent install.

That sequence is the point of the tool: one command to run a skill safely without leaving a permanent install behind.

## Next Steps

- [Famous Skills](/getting-started/famous-skills/) - try the most useful skills next
- [Official Examples](/examples/overview/) - browse the full set of runnable examples
- [Run Skills](/cli/run/) - learn the command in detail
- [FAQ](/getting-started/faq/) - understand when to stay in one-off mode and when to move deeper
- [Troubleshooting](/getting-started/troubleshooting/) - debug source resolution, agent detection, and scan gate failures
