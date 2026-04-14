# Security Policy

skillx is a security-focused tool that scans and gates agent skills before execution. We take the security of skillx itself seriously.

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.6.x   | Yes                |
| < 0.6   | No                 |

Only the latest release receives security updates. We recommend always running the most recent version.

## Reporting a Vulnerability

**Do not report security vulnerabilities through public GitHub issues.**

Instead, please report via GitHub Security Advisories:

- **GitHub Security Advisories**: [Report a vulnerability](https://github.com/skillx-run/skillx/security/advisories/new)

### What to Include

- A description of the vulnerability and its potential impact
- Step-by-step instructions to reproduce the issue
- Affected version(s)
- Any proof-of-concept code or logs, if available
- Your suggested fix, if you have one

### Response Timeline

- **Acknowledgment**: Within 48 hours of your report
- **Initial assessment**: Within 5 business days
- **Resolution target**: Critical issues within 14 days; others within 30 days

We will keep you informed of our progress throughout the process.

## Scope

The following are considered security issues in skillx:

- **Scanner bypass**: Crafted skill content that evades detection by the security scanner (MD, SC, RS rules)
- **Injection during skill processing**: Path traversal, zip-slip, or code injection during skill fetching, extraction, or injection
- **Unsafe file handling**: Writing files outside intended directories, symlink attacks, or TOCTOU race conditions
- **Credential exposure**: Leaking API tokens, GitHub credentials, or other secrets through logs, error messages, or cached data
- **Privilege escalation**: Gaining elevated permissions through skillx's session or agent injection mechanisms
- **Authentication/authorization flaws**: Bypassing scan gating (WARN/DANGER/BLOCK) or manipulating installed state

## Out of Scope

The following are **not** considered security issues in skillx:

- **Skill content**: The skills themselves are user-supplied content. skillx scans them and reports findings, but malicious skill content is the skill author's responsibility, not a skillx vulnerability
- **Local denial of service**: Resource exhaustion or crashes against the local CLI (skillx runs locally, not as a service)
- **Issues in third-party dependencies** without a demonstrated exploit path through skillx
- **Social engineering** of skillx users

## Disclosure Policy

We follow coordinated disclosure. Once a fix is available:

1. We release a patched version
2. We publish a GitHub Security Advisory with details
3. We credit the reporter (unless they prefer to remain anonymous)

## Credit

We gratefully acknowledge security researchers who report vulnerabilities responsibly. With your permission, we will credit you in:

- The GitHub Security Advisory
- The relevant CHANGELOG entry
- The project's security acknowledgments

If you prefer to remain anonymous, we will respect that preference.

## Contact

For security matters: [GitHub Security Advisories](https://github.com/skillx-run/skillx/security/advisories/new)

For general questions and bug reports: [GitHub Issues](https://github.com/skillx-run/skillx/issues)
