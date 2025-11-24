#!/usr/bin/env python3
"""
Automated test runner for GibberLink protocol test suite.
Provides comprehensive testing with reporting and CI/CD integration.
"""

import subprocess
import sys
import os
import json
from pathlib import Path
from datetime import datetime


def run_command(cmd, cwd=None):
    """Run command and return result."""
    try:
        result = subprocess.run(
            cmd,
            shell=True,
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=300  # 5 minute timeout
        )
        return result.returncode == 0, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return False, "", "Command timed out"
    except Exception as e:
        return False, "", str(e)


def install_dependencies():
    """Install test dependencies."""
    print("[PKG] Installing test dependencies...")
    success, stdout, stderr = run_command("pip install -r tests/requirements.txt")
    if not success:
        print(f"[FAIL] Failed to install dependencies: {stderr}")
        return False
    print("[OK] Dependencies installed")
    return True


def build_rust_library():
    """Build the Rust library with Python bindings."""
    print("[BUILD] Building Rust library with Python bindings...")
    success, stdout, stderr = run_command("cargo build --release", cwd="gibberlink-core")
    if not success:
        print(f"[FAIL] Failed to build Rust library: {stderr}")
        return False
    print("[OK] Rust library built")
    return True


def run_unit_tests():
    """Run unit tests."""
    print("[TEST] Running unit tests...")
    cmd = "pytest tests/ -m unit -v --tb=short"
    success, stdout, stderr = run_command(cmd)
    if success:
        print("[OK] Unit tests passed")
    else:
        print(f"[FAIL] Unit tests failed: {stderr}")
    return success, stdout


def run_integration_tests():
    """Run integration tests."""
    print("[TEST] Running integration tests...")
    cmd = "pytest tests/ -m integration -v --tb=short"
    success, stdout, stderr = run_command(cmd)
    if success:
        print("[OK] Integration tests passed")
    else:
        print(f"[FAIL] Integration tests failed: {stderr}")
    return success, stdout


def run_performance_tests():
    """Run performance benchmarks."""
    print("[TEST] Running performance tests...")
    cmd = "pytest tests/ -m performance -v --tb=short --benchmark-only"
    success, stdout, stderr = run_command(cmd)
    if success:
        print("[OK] Performance tests completed")
    else:
        print(f"[WARN] Performance tests had issues: {stderr}")
    return success, stdout


def run_robustness_tests():
    """Run robustness tests."""
    print("🛡️  Running robustness tests...")
    cmd = "pytest tests/ -m robustness -v --tb=short"
    success, stdout, stderr = run_command(cmd)
    if success:
        print("✅ Robustness tests passed")
    else:
        print(f"❌ Robustness tests failed: {stderr}")
    return success, stdout


def run_security_tests():
    """Run security validation tests."""
    print("[TEST] Running security tests...")
    cmd = "pytest tests/ -m security -v --tb=short"
    success, stdout, stderr = run_command(cmd)
    if success:
        print("[OK] Security tests passed")
    else:
        print(f"[FAIL] Security tests failed: {stderr}")
    return success, stdout


def run_full_coverage():
    """Run all tests with coverage reporting."""
    print("[TEST] Running full test suite with coverage...")
    cmd = "pytest tests/ --cov=gibberlink_core --cov-report=html --cov-report=json --cov-report=term-missing"
    success, stdout, stderr = run_command(cmd)
    if success:
        print("[OK] Full test suite completed")
        # Try to read coverage summary
        try:
            with open("coverage.json", "r") as f:
                coverage_data = json.load(f)
            total_coverage = coverage_data.get("totals", {}).get("percent_covered", 0)
            print(f"Coverage: {total_coverage:.1f}%")
        except Exception as e:
            print(f"[WARN] Could not read coverage data: {e}")
    else:
        print(f"[FAIL] Full test suite failed: {stderr}")
    return success, stdout


def generate_report(results):
    """Generate comprehensive test report."""
    print("📝 Generating test report...")

    report = {
        "timestamp": datetime.now().isoformat(),
        "summary": {
            "total_tests": sum(len(result[1].split("\n")) - 1 for result in results.values() if result[0]),
            "passed": sum(1 for result in results.values() if result[0]),
            "failed": sum(1 for result in results.values() if not result[0]),
            "categories": list(results.keys())
        },
        "details": results,
        "environment": {
            "python_version": sys.version,
            "platform": sys.platform,
            "working_directory": os.getcwd()
        }
    }

    # Save report
    report_file = "test_report.json"
    with open(report_file, "w") as f:
        json.dump(report, f, indent=2)

    print(f"✅ Report saved to {report_file}")

    # Print summary
    print("\n" + "="*50)
    print("TEST SUMMARY")
    print("="*50)
    print(f"Total categories: {len(results)}")
    print(f"Passed: {report['summary']['passed']}")
    print(f"Failed: {report['summary']['failed']}")
    print(".1f")

    if report['summary']['failed'] > 0:
        print("\n[FAIL] Some tests failed. Check the detailed report for more information.")
        return False
    else:
        print("\n[SUCCESS] All tests passed!")
        return True


def main():
    """Main test runner function."""
    print("[START] GibberLink Protocol Test Suite")
    print("="*40)

    # Setup
    if not install_dependencies():
        return 1

    if not build_rust_library():
        return 1

    # Run test categories
    test_results = {}

    # Unit tests
    success, output = run_unit_tests()
    test_results["unit"] = (success, output)

    # Integration tests
    success, output = run_integration_tests()
    test_results["integration"] = (success, output)

    # Performance tests
    success, output = run_performance_tests()
    test_results["performance"] = (success, output)

    # Robustness tests
    success, output = run_robustness_tests()
    test_results["robustness"] = (success, output)

    # Security tests
    success, output = run_security_tests()
    test_results["security"] = (success, output)

    # Full coverage run
    success, output = run_full_coverage()
    test_results["full_coverage"] = (success, output)

    # Generate report
    overall_success = generate_report(test_results)

    return 0 if overall_success else 1


if __name__ == "__main__":
    sys.exit(main())