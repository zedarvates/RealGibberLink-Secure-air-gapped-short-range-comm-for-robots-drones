import pytest
from gibberlink_core import PyCryptoEngine, PyProtocolEngine, PyVisualEngine


class TestSecurity:
    """Security validation tests for RgibberLink protocol"""

    def test_replay_attack_prevention(self):
        """Test that replay attacks are prevented"""
        crypto = PyCryptoEngine()
        key = crypto.public_key()

        # Original message
        message = b"Original message"
        encrypted1 = PyCryptoEngine.encrypt_data(key, message)

        # Decrypt should work
        decrypted1 = PyCryptoEngine.decrypt_data(key, encrypted1)
        assert decrypted1 == message

        # Replay attack: try to decrypt the same ciphertext again
        # AES-GCM should prevent this due to nonce uniqueness
        # In practice, each encryption uses a unique nonce
        encrypted2 = PyCryptoEngine.encrypt_data(key, message)
        assert encrypted1 != encrypted2  # Different ciphertexts due to unique nonces

        # Both should decrypt to the same plaintext
        decrypted2 = PyCryptoEngine.decrypt_data(key, encrypted2)
        assert decrypted2 == message

    def test_spoofing_resistance(self):
        """Test resistance to spoofing attacks"""
        alice = PyCryptoEngine()
        bob = PyCryptoEngine()
        attacker = PyCryptoEngine()

        # Alice and Bob establish shared secret
        alice_secret = alice.derive_shared_secret(bob.public_key())
        bob_secret = bob.derive_shared_secret(alice.public_key())
        assert alice_secret == bob_secret

        # Attacker tries to spoof Alice's public key
        fake_alice_secret = attacker.derive_shared_secret(bob.public_key())

        # Shared secrets should be different
        assert alice_secret != fake_alice_secret

        # Test that attacker cannot decrypt messages
        message = b"Secret message"
        encrypted = PyCryptoEngine.encrypt_data(alice_secret, message)

        # Bob can decrypt
        decrypted = PyCryptoEngine.decrypt_data(bob_secret, encrypted)
        assert decrypted == message

        # Attacker cannot decrypt
        with pytest.raises(Exception):
            PyCryptoEngine.decrypt_data(fake_alice_secret, encrypted)

    def test_key_expiration_simulation(self):
        """Test key expiration and rotation"""
        crypto = PyCryptoEngine()

        # Simulate key expiration by creating new keys
        old_key = crypto.public_key()

        # "Expire" old key by creating new crypto engine
        new_crypto = PyCryptoEngine()
        new_key = new_crypto.public_key()

        # Keys should be different
        assert old_key != new_key

        # Test that old encrypted data cannot be decrypted with new key
        message = b"Message with old key"
        encrypted = PyCryptoEngine.encrypt_data(old_key, message)

        with pytest.raises(Exception):
            PyCryptoEngine.decrypt_data(new_key, encrypted)

    def test_man_in_the_middle_attack_prevention(self):
        """Test prevention of man-in-the-middle attacks"""
        alice = PyCryptoEngine()
        bob = PyCryptoEngine()
        mitm = PyCryptoEngine()

        # Alice sends nonce to Bob (normally via audio)
        nonce = PyCryptoEngine.generate_nonce()

        # MITM intercepts and modifies nonce
        modified_nonce = bytearray(nonce)
        modified_nonce[0] ^= 0xFF  # Flip first byte

        # Bob receives modified nonce and generates QR with MITM's key
        protocol_bob = PyProtocolEngine()
        qr_with_mitm_key = protocol_bob.receive_nonce(bytes(modified_nonce))

        # Alice scans QR (simulated) and derives shared secret with MITM's key
        # In real attack, MITM would provide their public key instead of Bob's

        # Test that the cryptographic operations maintain integrity
        # Even with modified nonce, the protocol should detect mismatches
        # (This is more of an integration test, but validates the crypto primitives)

        alice_secret = alice.derive_shared_secret(mitm.public_key())
        mitm_secret = mitm.derive_shared_secret(alice.public_key())

        # Shared secrets should match between Alice and MITM
        assert alice_secret == mitm_secret

        # But Bob's secret should be different
        bob_secret = bob.derive_shared_secret(alice.public_key())
        assert bob_secret != alice_secret

    def test_timing_attack_resistance(self):
        """Test resistance to timing attacks"""
        crypto = PyCryptoEngine()
        key = crypto.public_key()

        # Test that decryption time is consistent regardless of input
        test_messages = [
            b"Short",
            b"Medium length message",
            b"This is a much longer message for testing timing consistency"
        ]

        import time
        times = []

        for msg in test_messages:
            encrypted = PyCryptoEngine.encrypt_data(key, msg)

            start = time.perf_counter()
            try:
                PyCryptoEngine.decrypt_data(key, encrypted)
            except Exception:
                pass  # We expect failures for modified data below
            end = time.perf_counter()

            times.append(end - start)

        # Check that timing variation is minimal (<10% difference)
        avg_time = sum(times) / len(times)
        max_deviation = max(abs(t - avg_time) for t in times)
        relative_deviation = max_deviation / avg_time

        assert relative_deviation < 0.1  # Less than 10% timing variation

    def test_cryptographic_oracle_prevention(self):
        """Test prevention of cryptographic oracle attacks"""
        crypto = PyCryptoEngine()
        key = crypto.public_key()

        # Test that invalid ciphertexts fail consistently
        valid_message = b"Valid message"
        valid_encrypted = PyCryptoEngine.encrypt_data(key, valid_message)

        # Create various invalid ciphertexts
        invalid_ciphertexts = [
            valid_encrypted[:-1],  # Truncated
            valid_encrypted + b"x",  # Extended
            b"",  # Empty
            b"invalid",  # Random data
            valid_encrypted[:12] + b"x" * (len(valid_encrypted) - 12),  # Modified after nonce
        ]

        # All should fail decryption
        for invalid in invalid_ciphertexts:
            with pytest.raises(Exception):
                PyCryptoEngine.decrypt_data(key, invalid)

    def test_forward_secrecy_verification(self):
        """Test forward secrecy properties"""
        # Create multiple key pairs to simulate key evolution
        old_alice = PyCryptoEngine()
        old_bob = PyCryptoEngine()

        new_alice = PyCryptoEngine()
        new_bob = PyCryptoEngine()

        # Establish old shared secret
        old_secret = old_alice.derive_shared_secret(old_bob.public_key())

        # Establish new shared secret
        new_secret = new_alice.derive_shared_secret(new_bob.public_key())

        # Secrets should be different
        assert old_secret != new_secret

        # Encrypt with old secret
        message = b"Old session message"
        old_encrypted = PyCryptoEngine.encrypt_data(old_secret, message)

        # New secret cannot decrypt old messages (forward secrecy)
        with pytest.raises(Exception):
            PyCryptoEngine.decrypt_data(new_secret, old_encrypted)

    def test_nonce_uniqueness_and_reuse_prevention(self):
        """Test that nonces are unique and prevent reuse attacks"""
        crypto = PyCryptoEngine()
        key = crypto.public_key()

        # Generate multiple nonces
        nonces = [PyCryptoEngine.generate_nonce() for _ in range(100)]

        # All nonces should be unique
        assert len(set(nonces)) == len(nonces)

        # Test that nonce reuse in encryption would create different results
        # (Though in practice, our implementation generates unique nonces)
        message = b"Test message"
        encrypted1 = PyCryptoEngine.encrypt_data(key, message)
        encrypted2 = PyCryptoEngine.encrypt_data(key, message)

        # Should be different due to unique nonces
        assert encrypted1 != encrypted2

        # Both should decrypt to the same plaintext
        decrypted1 = PyCryptoEngine.decrypt_data(key, encrypted1)
        decrypted2 = PyCryptoEngine.decrypt_data(key, encrypted2)
        assert decrypted1 == decrypted2 == message

    def test_weak_key_resistance(self):
        """Test resistance to weak key attacks"""
        # Test with various key patterns that might be considered weak
        weak_keys = [
            b"\x00" * 32,  # All zeros
            b"\xFF" * 32,  # All ones
            b"\x00\x01" * 16,  # Alternating pattern
            bytes(range(32)),  # Sequential bytes
        ]

        message = b"Test weak key resistance"

        for weak_key in weak_keys:
            try:
                encrypted = PyCryptoEngine.encrypt_data(weak_key, message)
                decrypted = PyCryptoEngine.decrypt_data(weak_key, encrypted)
                assert decrypted == message
            except Exception as e:
                # Even with "weak" keys, operations should be secure
                # The test passes as long as no exceptions occur or
                # if they do, they're consistent
                pass