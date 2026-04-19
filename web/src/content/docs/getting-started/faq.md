---
title: FAQ
description: Conceptual answers to the most common questions about first runs, scan gates, examples, and project usage.
---

## What is the difference between `run`, `scan`, and `install`?

Use the command that matches your stage:

- Use [`skillx run`](/cli/run/) when you want to try a skill right now for one task
- Use [`skillx scan`](/cli/scan/) when you want the security decision without launching an agent
- Use [`skillx install`](/cli/install/) when the skill has already proven useful and you want to keep it available

If you are still deciding whether a skill is worth keeping, `run` is the default. `install` is for persistence, not exploration.

## Why does skillx scan before inject?

Because the first useful control point is before the skill reaches your agent.

Once a skill is injected into the active agent environment, the risky content is already in the session. `scan before inject` gives you a chance to inspect findings, stop the run, or ask for more detail before the agent sees the files.

See [Security Overview](/security/overview/) for the full trust model.

## Does `PASS` mean the skill is safe?

No. `PASS` means the built-in analyzers did not detect known risky patterns.

That is a trust signal, not a guarantee. A skill can still be a bad fit for your environment, ask for more access than you want to give, or rely on behavior that static checks cannot fully predict.

If the stakes are high, use [`skillx scan`](/cli/scan/) first and read the skill source as well.

## Does `skillx run` permanently install anything?

No, not by default.

`run` is designed for temporary use: fetch, scan, inject, launch, clean up. That is why it is the right default for a first run.

If you want to keep a skill around after the session, use [`skillx install`](/cli/install/) or move into [Manage Project Skills](/guides/manage-project-skills/).

## When should I move from `run` to project-managed skills?

Move when the skill stops being a one-off experiment and starts becoming part of normal project workflow.

Good signs:

- You have already used the skill successfully more than once
- The team wants the same skill available across multiple sessions
- You want the source captured in `skillx.toml`
- You want updates and removals to be explicit project operations

That is the point where [Manage Project Skills](/guides/manage-project-skills/) becomes more useful than ad hoc `run`.

## What is the difference between Famous Skills and Official Examples?

They serve different jobs:

- [Famous Skills](/getting-started/famous-skills/) are curated external Agent Skills chosen for immediate usefulness
- [Official Examples](/examples/overview/) are stable first-party examples in this repository for learning structure and patterns

If you want the fastest convincing first run, start with Famous Skills. If you want to understand how skills are put together or copy a template, use Official Examples.

## Why can a Famous Skill change or disappear?

Because Famous Skills are external upstream content, not files controlled by this repository.

That is the tradeoff: they are more interesting and immediately useful, but they can move when the upstream repo changes. If you need a stable teaching example, prefer [Official Examples](/examples/overview/). If you need a stable production dependency, pin or manage a source you control.

## Which source format should I prefer?

Prefer the clearest stable source that points directly at a real skill.

In practice that usually means:

- a direct GitHub URL when you are copying from a repository page
- a `github:` source when you want a compact, script-friendly form
- a local path when you are iterating on your own skill

Legacy compatibility URLs may still resolve, but they are not the recommended discovery path.

## Should I start with a local example or an external Agent Skill?

Start with an external Agent Skill if your goal is to judge product value quickly.

Start with a local example if your goal is to learn skill structure or inspect every file closely.

That is why the docs split the main path this way:

- [First Run](/getting-started/first-run/) and [Famous Skills](/getting-started/famous-skills/) for immediate value
- [Official Examples](/examples/overview/) for learning and authoring patterns

## Where should I go if my question is not conceptual but operational?

Use the page that matches the kind of problem:

- [Troubleshooting](/getting-started/troubleshooting/) for install, first-run, agent detection, and source-resolution failures
- [Run Skills](/cli/run/) for lifecycle and launch behavior
- [Scan Skills](/cli/scan/) for risk thresholds and CI-style usage
- [Installation](/getting-started/installation/) for environment setup
