# Security Policy

## Supported Versions

Security updates are provided for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| main    | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in Capsule, please report it responsibly:

### Private Disclosure

**DO NOT** create a public GitHub issue for security vulnerabilities.

Instead, please report security issues via email to: **charlie@charlieroth.com**

### Information to Include

When reporting a vulnerability, please include:

1. **Description**: A clear description of the vulnerability
2. **Steps to Reproduce**: Detailed steps to reproduce the issue
3. **Impact**: Description of the potential impact
4. **Affected Components**: Which parts of the system are affected
5. **Suggested Fix**: If you have ideas for a fix (optional)

### What to Expect

1. **Acknowledgment**: We'll acknowledge receipt within 48 hours
2. **Assessment**: Initial assessment within 5 business days
3. **Updates**: Regular updates on investigation progress
4. **Resolution**: Coordinated fix and disclosure timeline

### Responsible Disclosure

We follow responsible disclosure practices:

- We'll work with you to understand and resolve the issue
- We'll provide credit for the discovery (if desired)
- We'll coordinate public disclosure after a fix is available
- We'll notify affected users appropriately

## Security Best Practices

When contributing to Capsule:

### Code Security
- Never commit secrets, API keys, or credentials
- Use secure coding practices for authentication and authorization
- Validate all user inputs
- Follow OWASP guidelines for web application security

### Database Security
- Use parameterized queries to prevent SQL injection
- Apply principle of least privilege for database access
- Encrypt sensitive data at rest and in transit

### Infrastructure Security
- Keep dependencies up to date
- Use secure communication channels (HTTPS/TLS)
- Follow container security best practices

## Security Features

Capsule implements the following security measures:

- JWT-based authentication
- Argon2 password hashing
- SQL injection prevention through SQLx
- Input validation and sanitization
- Secure session management

## Vulnerability Disclosure Timeline

We aim to:
- Acknowledge reports within 48 hours
- Provide initial assessment within 5 business days
- Release fixes within 30 days for critical vulnerabilities
- Release fixes within 90 days for non-critical vulnerabilities

Thank you for helping keep Capsule secure!
