# Contributing to RealGibber

Thank you for your interest in contributing to RealGibber! This document provides comprehensive guidelines for contributors to ensure high-quality, secure, and maintainable code for this mission-critical autonomous communication platform.

## Table of Contents
- [Introduction](#introduction)
- [Development Environment Setup](#development-environment-setup)
- [Code Style and Standards](#code-style-andstandards)
- [Testing Procedures](#testing-procedures)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting Guidelines](#issue-reporting-guidelines)
- [Security Considerations](#security-considerations)
- [Code of Conduct](#code-of-conduct)

## Introduction

RealGibber is a security-critical suite of directional communication protocols for autonomous systems. Contributions must prioritize security, reliability, and compliance with mission-critical standards. All contributors are expected to follow these guidelines to maintain the integrity of the platform.

### Getting Started

1. **Fork the Repository**: Create a personal fork of the RealGibber repository
2. **Clone Locally**: `git clone https://github.com/your-username/realgibber.git`
3. **Set Up Development Environment**: Follow the setup instructions below
4. **Create Feature Branch**: `git checkout -b feature/your-feature-name`

### Types of Contributions

- **Security Enhancements**: Cryptographic improvements, vulnerability fixes
- **Performance Optimizations**: Latency reductions, resource efficiency
- **Platform Support**: New hardware platforms, operating systems
- **Documentation**: API docs, tutorials, deployment guides
- **Testing**: Unit tests, integration tests, security tests
- **Bug Fixes**: Critical bug fixes with proper audit trails

## Development Environment Setup

### Prerequisites

- **Git** 2.30+ for version control
- **Visual Studio Code** with recommended extensions
- **Docker** (optional, for isolated testing environments)

### Rust Core Library Setup

#### System Requirements
```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install stable
rustup component add rustfmt clippy

# Verify installation
rustc --version  # Should be 1.70.0+
cargo --version
```

#### Build Dependencies
```bash
# Install system dependencies (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev

# Install system dependencies (macOS)
brew install openssl pkg-config

# Install system dependencies (Windows)
# Use vcpkg or install OpenSSL manually
```

#### Project Setup
```bash
cd rgibberlink-core

# Build project
cargo build --release

# Run tests
cargo test --release

# Generate documentation
cargo doc --open
```

### Android Development Setup

#### Android SDK and NDK
```bash
# Install Android SDK (automatic via script)
cd android-ndk
# Run install_android_sdk.bat on Windows
# Or follow manual installation steps below

# Manual SDK installation
export ANDROID_HOME=/path/to/android/sdk
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/25.2.9519653

# Add to PATH
export PATH=$PATH:$ANDROID_HOME/platform-tools:$ANDROID_HOME/tools/bin
```

#### Rust Android Targets
```bash
# Install Android targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android
rustup target add i686-linux-android
```

#### Android Studio Setup
```bash
# Install Android Studio
# Download from: https://developer.android.com/studio

# Configure project
cd android-app
./gradlew assembleDebug
```

### Web Development Setup

#### Node.js and TypeScript
```bash
# Install Node.js (version 18+)
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# Install TypeScript globally
npm install -g typescript

# Install dependencies
cd rgibberlink/hackathon_demo
npm install

# Start development server
npm run dev
```

#### WebAssembly Setup (Optional)
```bash
# Install wasm-pack
cargo install wasm-pack

# Build WebAssembly
wasm-pack build --target web --out-dir pkg
```

### Python Bindings Setup

#### Python Environment
```bash
# Install Python 3.8+
sudo apt-get install python3 python3-pip python3-venv

# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install build dependencies
pip install maturin

# Build bindings
cd rgibberlink-core
maturin develop --release

# Test Python interface
python3 -c "import gibberlink_core; print('Bindings working')"
```

## Code Style and Standards

### Rust Standards

#### Code Formatting
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

#### Linting
```bash
# Run clippy linter
cargo clippy -- -D warnings

# Specific security-focused checks
cargo clippy -- \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -D clippy::unimplemented \
  -D clippy::todo
```

#### Security Guidelines
```rust
// ✅ Good: Explicit error handling
match result {
    Ok(value) => process_data(value),
    Err(e) => {
        audit_log.record_error(&e);
        return Err(SecurityError::OperationFailed);
    }
}

// ❌ Bad: Using unwrap/expect
let data = result.unwrap(); // Dangerous in security-critical code
```

#### Memory Safety
- Use `zeroize` for sensitive data cleanup
- Avoid raw pointers unless absolutely necessary
- Implement proper drop traits for cryptographic keys
- Use bounds checking in array operations

### Kotlin/Java Standards (Android)

#### Code Style
```kotlin
// Follow Google Kotlin style guide
// Use official Android Kotlin extensions
// Implement proper null safety

class SecureCommunicationManager(
    private val context: Context,
    private val auditSystem: AuditSystem
) {
    // Use sealed classes for result types
    sealed class ConnectionResult {
        data class Success(val sessionId: String) : ConnectionResult()
        data class Failure(val error: SecurityError) : ConnectionResult()
    }

    // Implement proper error handling
    suspend fun establishSecureConnection(
        peerId: String,
        timeout: Duration = 30.seconds
    ): ConnectionResult {
        return try {
            val session = auditSystem.createAuditedSession(peerId)
            // Implementation...
            ConnectionResult.Success(session.id)
        } catch (e: SecurityException) {
            auditSystem.recordSecurityViolation(e)
            ConnectionResult.Failure(SecurityError.fromException(e))
        }
    }
}
```

#### Testing Requirements
- Minimum 80% code coverage
- Security-focused unit tests
- Integration tests for JNI interfaces
- Memory leak tests using LeakCanary

### TypeScript/JavaScript Standards (Web)

#### TypeScript Configuration
```json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true
  }
}
```

#### Security Best Practices
```typescript
// ✅ Good: Type-safe cryptographic operations
interface CryptoResult {
  success: true;
  data: Uint8Array;
} | {
  success: false;
  error: CryptoError;
}

async function secureEncrypt(
  data: Uint8Array,
  key: CryptoKey
): Promise<CryptoResult> {
  try {
    const encrypted = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv: generateIv() },
      key,
      data
    );
    return { success: true, data: new Uint8Array(encrypted) };
  } catch (error) {
    return { success: false, error: CryptoError.fromException(error) };
  }
}

// ❌ Bad: Loose typing
async function encrypt(data: any, key: any): Promise<any> {
  return crypto.subtle.encrypt(algorithm, key, data);
}
```

#### Code Quality Tools
```bash
# Type checking
npx tsc --noEmit

# Linting
npx eslint src/ --ext .ts,.tsx

# Testing
npm test

# Security audit
npm audit
npm run security-check
```

## Testing Procedures

### Test Categories

#### Unit Tests
```bash
# Rust unit tests
cargo test --lib

# Android unit tests
./gradlew testDebugUnitTest

# TypeScript unit tests
npm run test:unit
```

#### Integration Tests
```bash
# End-to-end communication tests
cargo test --test integration --features integration-tests

# Android instrumentation tests
./gradlew connectedAndroidTest

# Full stack tests
npm run test:e2e
```

#### Security Tests
```bash
# Cryptographic security tests
cargo test --test crypto_security

# Fuzz testing (where applicable)
cargo +nightly fuzz run fuzz_target

# Static security analysis
cargo audit
cargo +nightly udeps  # Check for unused dependencies
```

### Test Coverage Requirements

| Component | Minimum Coverage | Security Test Focus |
|-----------|------------------|-------------------|
| Rust Core | 85% | Cryptographic operations, memory safety |
| Android App | 80% | JNI interfaces, hardware access |
| Web Client | 75% | Input validation, XSS prevention |
| Python Bindings | 70% | Memory management, type safety |

### Performance Testing

```bash
# Benchmark cryptographic operations
cargo bench

# Memory usage profiling
cargo build --release
valgrind --tool=massif ./target/release/realgibber

# Latency measurements
./scripts/benchmark_latency.sh
```

## Pull Request Process

### Branch Naming Convention
```
feature/short-description
bugfix/issue-number-description
security/cve-description
hotfix/critical-fix-description
```

### Commit Message Standards
```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New features
- `fix`: Bug fixes
- `security`: Security-related changes
- `docs`: Documentation
- `test`: Testing
- `refactor`: Code refactoring
- `perf`: Performance improvements

**Examples:**
```
security(crypto): fix ECDH key exchange timing vulnerability

- Use constant-time operations for key comparison
- Add randomized delays to prevent timing attacks
- Update tests to verify timing attack resistance

Fixes: CVE-2024-XXXX
```

### PR Template Requirements

**Title:** `[COMPONENT] Brief description of changes`

**Description must include:**
- Problem statement
- Solution approach
- Security impact assessment
- Testing performed
- Breaking changes (if any)
- Related issues/PRs

### Review Process

#### Required Reviews
- **Security Review**: All changes affecting cryptography, authentication, or audit systems
- **Architecture Review**: Major API changes or structural modifications
- **Performance Review**: Changes affecting latency or resource usage

#### Automated Checks
```yaml
# .github/workflows/pr-checks.yml
name: PR Checks
on: [pull_request]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Security Audit
        run: cargo audit
      - name: Clippy Security
        run: cargo clippy -- -D clippy::unwrap_used

  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [stable, beta]
    steps:
      - uses: actions/checkout@v3
      - name: Test
        run: cargo test --all-features

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Coverage
        run: cargo tarpaulin --out Lcov
```

## Issue Reporting Guidelines

### Security Vulnerabilities
🚨 **NEVER report security vulnerabilities through public GitHub issues**

**Report Security Issues:**
1. Email: security@realgibber.com
2. Include: Detailed description, impact assessment, reproduction steps
3. Response: Within 24 hours for critical issues
4. Disclosure: Coordinated disclosure process

### Bug Reports
**Template:**
```markdown
## Bug Report

**Component:** [Rust Core/Android/Web/Python]

**Priority:** [Critical/High/Medium/Low]

**Environment:**
- OS: [e.g., Ubuntu 22.04]
- Version: [e.g., v0.3.0]
- Hardware: [e.g., Raspberry Pi 4]

**Description:**
Clear description of the issue

**Steps to Reproduce:**
1. Step 1
2. Step 2
3. Step 3

**Expected Behavior:**
What should happen

**Actual Behavior:**
What actually happens

**Security Impact:**
[High/Medium/Low/None] - Does this affect security?

**Additional Context:**
Logs, screenshots, configuration files
```

### Feature Requests
**Template:**
```markdown
## Feature Request

**Component:** [Rust Core/Android/Web/Python]

**Priority:** [Nice-to-have/Should-have/Must-have]

**Problem Statement:**
Current limitation or pain point

**Proposed Solution:**
Detailed feature description

**Security Considerations:**
How does this impact security?

**Alternative Solutions:**
Other approaches considered

**Additional Context:**
Use cases, requirements, dependencies
```

## Security Considerations for Contributors

### General Security Principles

1. **Defense in Depth**: Implement multiple layers of security controls
2. **Principle of Least Privilege**: Code should operate with minimal required permissions
3. **Fail-Safe Defaults**: Security controls should fail closed
4. **Audit Everything**: All security-relevant operations must be logged

### Cryptographic Standards

#### Approved Algorithms
- **Encryption**: AES-GCM (256-bit keys)
- **Key Exchange**: ECDH with Curve25519
- **Signatures**: Ed25519
- **Hashing**: SHA-256 (minimum)

#### Key Management
```rust
// ✅ Good: Secure key handling
use zeroize::Zeroize;

#[derive(Zeroize)]
#[zeroize(drop)]
struct SecureKey {
    data: Vec<u8>,
}

impl Drop for SecureKey {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}
```

### Input Validation
```rust
// ✅ Good: Strict input validation
fn validate_peer_id(peer_id: &str) -> Result<(), ValidationError> {
    if peer_id.len() < 8 || peer_id.len() > 64 {
        return Err(ValidationError::InvalidLength);
    }

    if !peer_id.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(ValidationError::InvalidCharacters);
    }

    Ok(())
}
```

### Secure Coding Practices

#### Avoid Common Vulnerabilities
- **Buffer Overflows**: Use bounds-checked operations
- **Injection Attacks**: Validate and sanitize all inputs
- **Timing Attacks**: Use constant-time operations
- **Race Conditions**: Proper synchronization primitives
- **Information Disclosure**: Don't leak sensitive data in logs

#### Memory Safety (Rust Specific)
```rust
// ✅ Good: Safe memory handling
fn process_secure_data(data: &[u8]) -> Result<SecureData, Error> {
    // Bounds checking is automatic
    if data.len() < HEADER_SIZE {
        return Err(Error::InsufficientData);
    }

    // Safe slicing
    let header = &data[..HEADER_SIZE];
    let payload = &data[HEADER_SIZE..];

    // Ownership prevents use-after-free
    Ok(SecureData::from_parts(header, payload))
}
```

### Contributing Security Improvements

1. **Vulnerability Research**: Document findings privately
2. **Patch Development**: Follow secure coding guidelines
3. **Testing**: Include security test cases
4. **Documentation**: Update security documentation
5. **Review**: Request security-focused code review

## Code of Conduct

### Our Pledge

We pledge to make participation in RealGibber a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Standards

**Acceptable Behavior:**
- Using welcoming and inclusive language
- Respecting differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

**Unacceptable Behavior:**
- Harassment, intimidation, or discrimination
- Personal attacks or trolling
- Publishing private information without permission
- Any conduct which could reasonably be considered inappropriate

### Responsibilities

**Project Maintainers:**
- Enforce the Code of Conduct
- Investigate complaints promptly and fairly
- Take appropriate corrective action
- Maintain confidentiality

**Community Members:**
- Adhere to the Code of Conduct
- Report unacceptable behavior
- Assist in creating a positive environment

### Enforcement

Violations will be dealt with appropriately, up to and including permanent expulsion from the community. Reports can be made by contacting the project maintainers at conduct@realgibber.com.

### Attribution

This Code of Conduct is adapted from the [Contributor Covenant](https://www.contributor-covenant.org/), version 2.0.

---

**Thank you for contributing to RealGibber and helping make autonomous systems more secure and reliable!** 🚀