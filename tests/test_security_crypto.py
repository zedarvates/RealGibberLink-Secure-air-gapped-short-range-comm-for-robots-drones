#!/usr/bin/env python3
"""
Security Tests for GibberLink Cryptographic Components

Tests ECDH key exchange, AES-GCM encryption, HMAC validation,
short-lived keys, and signed logging integrity.
"""

import pytest
import time
from datetime import datetime
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import ed25519


def test_ephemeral_key_exchange():
    """Test ECDH ephemeral key derivation and short validity"""

    from gibberlink_core import CryptoEngine, EphemeralKeySession

    # Test key exchange between two engines
    engine1 = CryptoEngine()
    engine2 = CryptoEngine()

    # Get public keys
    pub1 = engine1.ecdh_public_key()
    pub2 = engine2.ecdh_public_key()

    # Derive shared secrets
    session1 = engine1.derive_ephemeral_shared_secret(pub2)
    session2 = engine2.derive_ephemeral_shared_secret(pub1)

    # Verify keys are identical
    assert session1.key() == session2.key()

    # Test short validity (≤5 seconds)
    assert not session1.is_expired()  # Should be valid immediately
    time.sleep(5.1)  # Wait past expiration
    assert session1.is_expired()  # Should expire after 5 seconds

    # Test post-usage invalidation
    session1.invalidate()
    assert session1.is_expired()


def test_dual_channel_encryption():
    """Test AES-GCM for IR and HMAC-SHA256 for ultrasonic channels"""

    from gibberlink_core import CryptoEngine

    key = b'0' * 32  # 256-bit key
    payload = b'Test message for encryption'
    timestamp = 1640995200  # Unix timestamp

    # Test IR payload encryption (AES-GCM)
    encrypted_ir = CryptoEngine.encrypt_ir_payload(key, payload, timestamp)
    decrypted_ir = CryptoEngine.decrypt_ir_payload(key, encrypted_ir)
    assert decrypted_ir == payload

    # Test ultrasonic frame encryption (HMAC-SHA256)
    hmac_frame = CryptoEngine.encrypt_ultrasonic_frame(key, payload, timestamp)
    assert len(hmac_frame) == 32  # SHA256 output length

    # Test HMAC verification
    assert CryptoEngine.verify_ultrasonic_frame(key, payload, timestamp, hmac_frame)
    # Test verification failure with wrong key
    assert not CryptoEngine.verify_ultrasonic_frame(b'1' * 32, payload, timestamp, hmac_frame)


def test_device_fingerprinting():
    """Test hardware fingerprint generation (non-tracking)"""

    from gibberlink_core import CryptoEngine

    # Same device info should produce same fingerprint
    device_info1 = b'hardware_id:ABC123,cpu:arm64,os:android'
    device_info2 = b'hardware_id:ABC123,cpu:arm64,os:android'
    device_info3 = b'hardware_id:XYZ789,cpu:x86,os:linux'

    fingerprint1 = CryptoEngine.generate_device_fingerprint(device_info1)
    fingerprint2 = CryptoEngine.generate_device_fingerprint(device_info2)
    fingerprint3 = CryptoEngine.generate_device_fingerprint(device_info3)

    assert fingerprint1 == fingerprint2  # Same device info
    assert fingerprint1 != fingerprint3  # Different device info
    assert len(fingerprint1) == 32  # SHA256 output length


def test_signed_logging():
    """Test append-only signed logging with integrity verification"""

    from gibberlink_core import SignedLogger, LogEvent

    # Create logger
    crypto = CryptoEngine()
    device_id = b'test_device_123'
    logger = SignedLogger(crypto, device_id, 100)

    # Test logging events
    event1 = LogEvent.SessionStarted(**{
        "peer_fingerprint": b'A' * 32,
        "timestamp": time.time(),
    })

    event2 = LogEvent.MessageSent(**{
        "sequence_id": 1,
        "size_bytes": 256,
    })

    # Log events
    logger.log_event(event1)
    logger.log_event(event2)

    assert logger.log_entries.len() == 2

    # Test integrity verification
    assert logger.verify_log_integrity()

    # Test tamper detection
    logger.log_entries[0].sequence_number = 999  # Tamper with sequence
    assert not logger.verify_log_integrity()  # Should detect tampering


def test_cross_channel_validation():
    """Test coupled nonce and MAC validation"""

    from gibberlink_core import ChannelValidator, ChannelData, ChannelType, ValidationConfig

    # Create validator
    config = ValidationConfig(
        temporal_tolerance_ms=100,
        quality_threshold=0.8,
    )
    validator = ChannelValidator(config)

    # Test temporal coupling (within window)
    now = time.time() * 1000000  # microseconds

    laser_data = ChannelData(
        channel_type=ChannelType.Laser,
        data=b'laser_payload',
        timestamp=now,
        sequence_id=1,
    )

    ultrasound_data = ChannelData(
        channel_type=ChannelType.Ultrasonic,
        data=b'ultrasound_payload',
        timestamp=now + 50000,  # 50ms difference (within 100ms window)
        sequence_id=1,
    )

    # This should work in a full implementation, but we'll test the temporal aspect
    assert validator.validate_temporal_coupling(laser_data, ultrasound_data)

    # Test temporal coupling failure (outside window)
    ultrasound_late = ChannelData(
        channel_type=ChannelType.Ultrasonic,
        data=b'ultrasound_payload',
        timestamp=now + 150000,  # 150ms difference (>100ms window)
        sequence_id=1,
    )

    assert not validator.validate_temporal_coupling(laser_data, ultrasound_late)


def test_anti_replay_protection():
    """Test nonce replay prevention with coupled data"""

    from gibberlink_core import ChannelValidator, ChannelData, ChannelType

    validator = ChannelValidator()

    laser_data = ChannelData(
        channel_type=ChannelType.Laser,
        data=b'nonce123',
        timestamp=time.time(),
        sequence_id=1,
    )

    ultrasound_data = ChannelData(
        channel_type=ChannelType.Ultrasonic,
        data=b'challenge456',
        timestamp=time.time(),
        sequence_id=1,
    )

    # First validation should pass
    assert validator.validate_anti_replay(laser_data, ultrasound_data)

    # Second validation with same data should fail (replay)
    assert not validator.validate_anti_replay(laser_data, ultrasound_data)


def test_constant_time_comparison():
    """Test timing attack resistant comparison"""

    from gibberlink_core import CryptoEngine

    # Same data should compare equal
    assert CryptoEngine.constant_time_eq(b"test123", b"test123")

    # Different lengths should not compare equal
    assert not CryptoEngine.constant_time_eq(b"test123", b"test12")
    assert not CryptoEngine.constant_time_eq(b"test123", b"test124")

    # Different data should not be equal
    assert not CryptoEngine.constant_time_eq(b"test123", b"other123")


def test_secure_random_generation():
    """Test cryptographically secure random bytes"""

    from gibberlink_core import CryptoEngine

    # Test different sizes
    rand8 = CryptoEngine.generate_secure_random_bytes(8)
    rand32 = CryptoEngine.generate_secure_random_bytes(32)
    rand128 = CryptoEngine.generate_secure_random_bytes(128)

    assert len(rand8) == 8
    assert len(rand32) == 32
    assert len(rand128) == 128

    # Test randomness (adjacent bytes should be different with high probability)
    # This is a statistical test - in a real test we'd use proper randomness testing
    assert any(a != b for a, b in zip(rand128, rand128[1:]))


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
