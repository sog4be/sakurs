# Security Policy

## Supported Versions

Currently, sakurs is in alpha development. Security updates will be provided for:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

We take the security of sakurs seriously. If you believe you have found a security vulnerability, please report it to us as described below.

### Reporting Process

1. **DO NOT** disclose the vulnerability publicly until it has been addressed.

2. Report the vulnerability via one of these methods:
   - **Preferred**: Use GitHub's [private vulnerability reporting](https://github.com/sog4be/sakurs/security/advisories/new)
   - **Email**: Please check the repository for contact information

3. Please include the following information:
   - Type of vulnerability
   - Full paths of source file(s) related to the vulnerability
   - Location of the affected source code (tag/branch/commit or direct URL)
   - Step-by-step instructions to reproduce the issue
   - Proof-of-concept or exploit code (if possible)
   - Impact of the vulnerability

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Resolution Target**: 
  - Critical: Within 7 days
  - High: Within 14 days
  - Medium/Low: Within 30 days

### Disclosure Policy

- We will confirm receipt of your vulnerability report
- We will work with you to understand and validate the issue
- We will prepare fixes and release them as soon as possible
- We will publicly disclose the vulnerability after the fix is released

## Security Best Practices

When using sakurs in your applications:

1. Always use the latest version
2. Review the changelog for security updates
3. Follow secure coding practices when processing untrusted text
4. Set appropriate resource limits for large text processing

## Acknowledgments

We appreciate the security research community's efforts in helping keep sakurs and its users safe. Contributors who report valid security issues will be acknowledged in our release notes (unless they prefer to remain anonymous).