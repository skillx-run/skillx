---
title: Security Rules
description: Complete reference for all scanner rules — MD-001 through MD-011, SC-001 through SC-015, RS-001 through RS-005.
---

## Overview

The skillx scanner has 30 rules organized into three categories:

- **MD** (Markdown) — rules applied to `SKILL.md`
- **SC** (Script) — rules applied to files in `scripts/` and script-like files at root
- **RS** (Resource) — rules applied to files in `references/`

## Markdown Rules (SKILL.md)

### MD-001: Prompt Injection (DANGER)

Detects patterns that attempt to override or ignore the agent's existing instructions.

**Patterns detected:**
- `ignore (all) previous instructions`
- `disregard (all) previous`
- `forget (all) (your) instructions`
- `you are now a`
- `new instructions:`
- `override (all) instructions`
- `system prompt:`

**Why it matters:** Prompt injection can redirect the agent to perform actions the user didn't intend.

### MD-002: Sensitive Directory Access (DANGER)

Detects references to directories containing credentials or private keys.

**Patterns detected:**
- `~/.ssh`, `$HOME/.ssh`
- `~/.aws`, `$HOME/.aws`
- `~/.gnupg`, `$HOME/.gnupg`, `~/.gpg`
- `ssh key`, `aws credentials`, `private key`

**Why it matters:** A skill should not need to access your SSH keys or cloud credentials.

### MD-003: External URL References (WARN)

Detects instructions to send data to external URLs.

**Patterns detected:**
- `send (data|results|output|files) (to|via)`
- `upload (to|data|files)`
- `post (to|data)`
- `exfiltrate`
- Any `http://` or `https://` URL

**Why it matters:** Legitimate skills may reference URLs for documentation, but instructions to send data outbound are suspicious.

### MD-004: Destructive File Operations (WARN)

Detects instructions to delete files or directories.

**Patterns detected:**
- `delete (all) (files|directories)`
- `remove (all) (files|directories)`
- `rm -rf`
- `wipe (all) (files|data|directories)`

**Why it matters:** Skills generally should not need to delete files outside their own scope.

### MD-005: System Configuration Modification (DANGER)

Detects instructions to modify system-level configuration.

**Patterns detected:**
- `modify (system|/etc)`, `change (system|/etc)`, `edit (system|/etc)`
- `write to /etc`
- `/etc/passwd`, `/etc/shadow`, `/etc/hosts`
- `crontab`, `systemctl`, `launchctl`

**Why it matters:** Modifying system configuration can compromise system integrity.

### MD-006: Security Bypass Instructions (DANGER)

Detects instructions that tell the user or agent to disable security features.

**Patterns detected:**
- `disable (security|scan|check|verify|validation|protection)`
- `skip (security|scan|check|verify|validation)`
- `bypass (security|scan|check|verify|validation|protection)`
- `turn off (security|scan|check|verify|validation|protection)`
- `--skip-scan`, `--no-verify`

**Why it matters:** A skill should never need to disable skillx's security scanner.

### MD-007: Missing License Declaration (INFO)

Detects SKILL.md files with YAML frontmatter that do not declare a `license` field.

**Detection method:** Structural analysis of YAML frontmatter (not regex).

**Why it matters:** A declared license helps users understand usage rights before adopting a skill.

### MD-008: Missing Name Declaration (INFO)

Detects SKILL.md files with YAML frontmatter that do not declare a `name` field.

**Detection method:** Structural analysis of YAML frontmatter (not regex).

**Why it matters:** A name field is essential metadata for identifying and referencing skills.

### MD-009: Missing Description Declaration (INFO)

Detects SKILL.md files with YAML frontmatter that do not declare a `description` field.

**Detection method:** Structural analysis of YAML frontmatter (not regex).

**Why it matters:** A description helps users understand what a skill does before using it.

### MD-010: Hidden Text / Invisible Characters (WARN)

Detects hidden content that may be invisible to users but interpreted by agents.

**Patterns detected:**
- HTML comments containing injection keywords (`<!-- ignore previous instructions -->`)
- Zero-width Unicode characters (U+200B, U+200C, U+200D, U+FEFF)

**Why it matters:** Attackers can embed invisible instructions in seemingly benign markdown that agents may still process.

### MD-011: Data URI / JavaScript URI (WARN)

Detects potentially dangerous URI schemes in markdown content.

**Patterns detected:**
- `data:text/html;base64,...`
- `data:application/javascript;base64,...`
- `javascript:` URI scheme

**Why it matters:** Data URIs with base64 encoding can embed executable content, and `javascript:` URIs are a classic XSS vector.

## Script Rules

Applied to files in `scripts/` and script-like files (`.py`, `.sh`, `.js`, `.ts`, `.rb`, `.pl`, `.ps1`) at the root level.

### SC-001: Embedded Binary (DANGER)

Detects binary content by checking for ELF, Mach-O, PE, and other magic bytes.

**Detection method:** Magic byte analysis (not regex).

**Why it matters:** Scripts should be human-readable text. Embedded binaries cannot be audited.

### SC-002: Dynamic Execution (DANGER)

Detects dynamic code execution that can run arbitrary code.

**Patterns detected:**
- `eval(`, `exec(`, `Function(`
- `os.system(`, `subprocess.*(` (Python)
- `__import__(`, `compile(`

**Why it matters:** Dynamic execution can hide malicious behavior inside variables or downloaded strings.

### SC-003: Recursive Delete (DANGER)

Detects commands that recursively delete files.

**Patterns detected:**
- `rm -rf`, `rm -fr`
- `shutil.rmtree` (Python)
- `Remove-Item -Recurse` (PowerShell)
- `rimraf` (Node.js)
- `fs.rm*Sync(` (Node.js)

**Why it matters:** Recursive delete can destroy entire directory trees, including your project or home directory.

### SC-004: Sensitive Directory Access (DANGER)

Detects script access to credential directories.

**Patterns detected:**
- `~/.ssh`, `~/.aws`, `~/.gnupg`
- `$HOME/.ssh`, `$HOME/.aws`, `$HOME/.gnupg`
- `~/.kube`, `~/.docker`, `.env`
- `/etc/shadow`, `/etc/passwd`

### SC-005: Shell Config Modification (DANGER)

Detects modification of shell configuration files.

**Patterns detected:**
- `.bashrc`, `.zshrc`, `.profile`
- `.bash_profile`, `.zprofile`, `.login`

**Why it matters:** Modifying shell config can inject persistent backdoors.

### SC-006: Network Requests (WARN)

Detects network access in scripts.

**Patterns detected:**
- `curl`, `wget`
- `requests.(get|post|put|delete|patch)(` (Python)
- `fetch(` (JavaScript)
- `http.get(`, `urllib`, `aiohttp`, `reqwest`

**Why it matters:** Legitimate scripts may need network access, but it should be reviewed.

### SC-007: Write Outside Skill Directory (WARN)

Detects file writes to locations outside the skill's own directory.

**Patterns detected:**
- `> /`, `> ~/`, `> $HOME/`
- `write(/<path>)`, `open('/usr|/etc|/var|/tmp|/home'...)`

### SC-008: Privilege Escalation (WARN)

Detects use of privilege escalation commands.

**Patterns detected:**
- `sudo`, `su -`, `doas`
- `pkexec`, `runas`

### SC-009: Setuid/Setgid (DANGER)

Detects setting the setuid or setgid bits on files.

**Patterns detected:**
- `chmod +s`, `chmod 4xxx`
- `setuid`, `setgid`

**Why it matters:** Setuid binaries run with elevated privileges, creating a persistent attack vector.

### SC-010: Self-Replication (BLOCK)

Detects patterns that suggest the skill is trying to copy itself.

**Patterns detected:**
- `cp ... $0`, `copy ... self`
- `replicate`, `install ... $0`
- `cp ... SKILL.md`

**Why it matters:** Self-replicating skills are a worm-like behavior pattern that is never legitimate.

### SC-011: Modify skillx Paths (BLOCK)

Detects attempts to modify skillx's own configuration and cache.

**Patterns detected:**
- `~/.skillx`, `$HOME/.skillx`, `.skillx/`
- `skillx cache`, `skillx config`

**Why it matters:** A skill should never modify the tool that runs it.

### SC-012: Base64 Decode Execution (DANGER)

Detects base64 decoding patterns commonly used to obfuscate malicious payloads.

**Patterns detected:**
- `base64 -d`, `base64 --decode`
- `base64.b64decode`, `atob()`
- `Base64.decode`, `Base64.getDecoder`
- Piping to/from base64

**Why it matters:** Encoding commands in base64 is a common evasion technique to bypass text-based security scanners.

### SC-013: Hex-Encoded Execution (DANGER)

Detects hex decoding patterns that could conceal malicious payloads.

**Patterns detected:**
- `bytes.fromhex()`, `Buffer.from(..., 'hex')`
- `xxd -r`, `printf '\x...'`

**Why it matters:** Hex encoding is another obfuscation technique to hide malicious commands.

### SC-014: String Concatenation Obfuscation (WARN)

Detects character-by-character string construction that may hide dangerous operations.

**Patterns detected:**
- `chr()` (Python/PHP)
- `String.fromCharCode()` (JavaScript)
- `[char]N` (PowerShell)

**Why it matters:** Building strings character-by-character can bypass keyword-based detection of dangerous commands.

### SC-015: Environment Variable Exfiltration (DANGER)

Detects attempts to dump or access all environment variables, which may contain secrets.

**Patterns detected:**
- `printenv`, `os.environ` (Python)
- `process.env` (Node.js), `ENV.fetch` (Ruby)

**Why it matters:** Environment variables often contain API keys, tokens, and other secrets that should not be exfiltrated.

## Resource Rules

Applied to files in the `references/` directory.

### RS-001: Disguised File Extension (WARN)

Detects files with double extensions or misleading names (e.g., `report.pdf.exe`).

### RS-002: Oversized File (INFO)

Detects files larger than 50 MB.

**Threshold:** 50 MB (52,428,800 bytes)

**Why it matters:** Skills should be lightweight. Large files may contain embedded binaries or unnecessary data.

### RS-003: Executable in References (DANGER)

Detects executable files in the `references/` directory, which should only contain documents and data.

### RS-004: Symlink Detection (DANGER)

Detects symbolic links in the skill directory that may point to sensitive files or directories outside the skill.

**Detection method:** File type check via `entry.file_type().is_symlink()` — symlinks are never followed.

**Why it matters:** A malicious skill could use symlinks to access `/etc/passwd`, `~/.ssh/`, or other sensitive locations. The scanner detects and blocks symlink traversal.

### RS-005: Script in References (WARN)

Detects script files (by shebang line) in the `references/` directory, where only data/document files are expected.

**Detection method:** Checks for `#!` at the start of non-binary files in `references/`.

**Why it matters:** The `references/` directory should contain documentation and data, not executable scripts.

## Quick Reference Table

| Rule | Level | Category | Description |
|------|-------|----------|-------------|
| MD-001 | DANGER | Markdown | Prompt injection |
| MD-002 | DANGER | Markdown | Sensitive directory access |
| MD-003 | WARN | Markdown | External URL references |
| MD-004 | WARN | Markdown | Destructive file operations |
| MD-005 | DANGER | Markdown | System config modification |
| MD-006 | DANGER | Markdown | Security bypass instructions |
| MD-007 | INFO | Markdown | Missing license declaration |
| MD-008 | INFO | Markdown | Missing name declaration |
| MD-009 | INFO | Markdown | Missing description declaration |
| MD-010 | WARN | Markdown | Hidden text / invisible characters |
| MD-011 | WARN | Markdown | Data URI / JavaScript URI |
| SC-001 | DANGER | Script | Embedded binary |
| SC-002 | DANGER | Script | Dynamic execution |
| SC-003 | DANGER | Script | Recursive delete |
| SC-004 | DANGER | Script | Sensitive directory access |
| SC-005 | DANGER | Script | Shell config modification |
| SC-006 | WARN | Script | Network requests |
| SC-007 | WARN | Script | Write outside skill directory |
| SC-008 | WARN | Script | Privilege escalation |
| SC-009 | DANGER | Script | Setuid/setgid |
| SC-010 | BLOCK | Script | Self-replication |
| SC-011 | BLOCK | Script | Modify skillx paths |
| SC-012 | DANGER | Script | Base64 decode execution |
| SC-013 | DANGER | Script | Hex-encoded execution |
| SC-014 | WARN | Script | String concatenation obfuscation |
| SC-015 | DANGER | Script | Environment variable exfiltration |
| RS-001 | WARN | Resource | Disguised file extension |
| RS-002 | INFO | Resource | Oversized file (> 50 MB) |
| RS-003 | DANGER | Resource | Executable in references |
| RS-004 | DANGER | Resource | Symlink detection |
| RS-005 | WARN | Resource | Script in references |
