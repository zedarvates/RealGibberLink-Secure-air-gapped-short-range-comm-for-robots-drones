# Contributing to RealGibber

Thank you for your interest in contributing to RealGibber! This document provides guidelines for contributors.

## Quick Start

1. **Fork and Clone**: Fork the repository and clone it locally
2. **Setup Environment**: Follow the detailed setup in `Documentations/CONTRIBUTING.md`
3. **Create Branch**: Use `feature/`, `bugfix/`, or `security/` prefixes
4. **Make Changes**: Follow code style guidelines
5. **Test**: Run all tests and ensure security compliance
6. **Submit PR**: Create a pull request with detailed description

## Key Guidelines

- **Security First**: All changes must maintain or improve security
- **Code Quality**: Follow Rust/Android/TypeScript standards
- **Testing**: Minimum 80% coverage, security-focused tests
- **Documentation**: Update docs for any API changes

## Detailed Instructions

For comprehensive contributing guidelines, including setup instructions, code standards, testing procedures, and security considerations, please see [`Documentations/CONTRIBUTING.md`](Documentations/CONTRIBUTING.md).

## Security Issues

🚨 **Never report security vulnerabilities through public issues**

Report security issues to: security@realgibber.com

## Code of Conduct

Please review our [Code of Conduct](Documentations/CONTRIBUTING.md#code-of-conduct) in the detailed contributing guide.

---

**Thank you for helping make RealGibber more secure and reliable!** 🚀

## Code Formatting and Style Guidelines

### Rust Code Style

- Use `rustfmt` for automatic code formatting:
  ```bash
  cargo fmt
  ```
- Follow Rust naming conventions (snake_case for functions/variables, CamelCase for types)
- Use `clippy` for linting and follow its suggestions:
  ```bash
  cargo clippy
  ```
- Maximum line length: 100 characters
- Use 4 spaces for indentation

### Android/Kotlin Code Style

- Follow Android Kotlin style guide
- Use standard Android naming conventions
- Run `./gradlew ktlintCheck` for linting
- Format code with `./gradlew ktlintFormat`

### TypeScript/JavaScript Code Style

- Use ESLint and Prettier for code quality
- Follow Airbnb JavaScript style guide
- Run `npm run lint` to check code quality
- Use TypeScript strict mode enabled

### General Guidelines

- Write clear, concise comments for complex logic
- Use meaningful variable and function names
- Follow the principle of least privilege
- Avoid magic numbers; use named constants

## Testing Requirements and Procedures

### Testing Standards

- **Minimum Test Coverage**: All new code must maintain at least 80% test coverage
- **Security Testing**: All changes must include security-focused unit and integration tests
- **Performance Testing**: Critical path changes require performance benchmarks

### Running Tests

#### Rust Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

#### Android Tests
```bash
# Run unit tests
./gradlew test

# Run instrumented tests
./gradlew connectedAndroidTest
```

#### Python Tests
```bash
# Install dependencies
pip install -r requirements.txt

# Run tests
python -m pytest

# Run with coverage
python -m pytest --cov=realgibber --cov-report=html
```

### Writing Tests

- Write unit tests for all public functions
- Include integration tests for cross-component interactions
- Add security tests for cryptographic functions
- Use property-based testing where applicable

## Security Contribution Guidelines

### Security-First Development

- **Security Review**: All changes undergo security review before merging
- **Vulnerability Disclosure**: Report security issues to security@realgibber.com
- **Secure Coding**: Follow OWASP guidelines for secure coding practices

### Security Checklist for Contributors

- [ ] No hardcoded secrets or keys
- [ ] Input validation and sanitization implemented
- [ ] Proper error handling without information leakage
- [ ] Cryptographic operations use approved algorithms
- [ ] Access controls properly implemented
- [ ] Dependencies scanned for vulnerabilities

### Security Testing Requirements

- New features must include security test cases
- All cryptographic code must be reviewed by security team
- Changes to authentication/authorization require additional scrutiny

## Code Review Process

### Review Workflow

1. **Submit PR**: Create pull request with clear description
2. **Automated Checks**: CI/CD pipeline runs automatically
3. **Peer Review**: 2+ reviewers required for all changes
4. **Security Review**: Security team reviews for critical changes
5. **Approval**: Maintainers approve before merge

### Review Criteria

#### Code Quality
- [ ] Code follows style guidelines
- [ ] Tests included and passing
- [ ] Documentation updated
- [ ] No security vulnerabilities

#### Security Review
- [ ] Security checklist completed
- [ ] No sensitive data exposure
- [ ] Secure coding practices followed
- [ ] Threat model updated if needed

#### Performance
- [ ] No performance regressions
- [ ] Benchmarks pass if applicable
- [ ] Memory safety maintained

### Review Process Timeline

- **Initial Review**: Within 2 business days
- **Security Review**: Within 3 business days for security-related changes
- **Merge**: After all reviews pass and CI/CD succeeds

## CI/CD Workflow Explanation

### GitHub Actions Pipeline

Our CI/CD pipeline consists of the following stages:

#### 1. Code Quality Checks
- **Linting**: Runs clippy, ESLint, ktlint
- **Formatting**: Verifies code formatting with rustfmt, Prettier
- **Security Scanning**: Uses cargo-audit and safety for dependency vulnerabilities

#### 2. Testing
- **Unit Tests**: Run in parallel across all components
- **Integration Tests**: Test cross-component interactions
- **Security Tests**: Validate cryptographic implementations
- **Performance Tests**: Run benchmarks and compare against baselines

#### 3. Build
- **Rust Build**: Cross-compilation for multiple targets
- **Android Build**: APK generation for debug and release
- **WebAssembly Build**: WASM module compilation

#### 4. Security Analysis
- **Static Analysis**: CodeQL and other static analysis tools
- **Dependency Scanning**: Automated vulnerability detection
- **Binary Analysis**: Security assessment of compiled artifacts

#### 5. Deployment
- **Staging**: Automatic deployment to staging environment
- **Release**: Manual approval required for production deployment

### Branch Protection Rules

- **main branch**: Requires 2 approvals, passing CI/CD, security review
- **develop branch**: Requires 1 approval, passing CI/CD
- **feature branches**: CI/CD required, peer review recommended

### Contributing to CI/CD

When adding new CI/CD features:
1. Update `.github/workflows/` files
2. Test locally with `act` or similar tools
3. Document any new requirements
4. Ensure security compliance
