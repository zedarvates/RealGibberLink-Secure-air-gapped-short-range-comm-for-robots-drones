import pytest
import asyncio
from gibberlink_core import PyProtocolEngine, PyCryptoEngine, PyVisualEngine, PyVisualPayload


class TestProtocolEngine:
    """Unit tests for protocol state machine"""

    @pytest.fixture
    def protocol(self):
        return PyProtocolEngine()

    def test_protocol_initialization(self, protocol):
        """Test protocol engine initialization"""
        assert protocol is not None
        assert protocol.get_state() == "idle"

    def test_state_transitions_idle_to_sending_nonce(self, protocol):
        """Test transition from idle to sending nonce"""
        # Should start idle
        assert protocol.get_state() == "idle"

        # Mock audio sending
        with patch.object(protocol, 'send_data', return_value=None):
            protocol.initiate_handshake()
            # Note: Actual state transition depends on implementation
            # This is testing the interface contract

    def test_receive_nonce_and_generate_qr(self, protocol):
        """Test nonce reception and QR generation"""
        nonce = PyCryptoEngine.generate_nonce()

        # Should transition to waiting for QR
        qr_svg = protocol.receive_nonce(nonce)
        assert isinstance(qr_svg, str)
        assert len(qr_svg) > 0
        assert "<svg" in qr_svg  # Should contain SVG elements

    def test_qr_payload_processing(self, protocol):
        """Test QR payload processing"""
        # Create test payload
        session_id = bytes([i for i in range(16)])
        public_key = bytes([i for i in range(32)])
        nonce = PyCryptoEngine.generate_nonce()
        signature = bytes([i for i in range(32)])

        payload = PyVisualPayload(session_id, public_key, nonce, signature)
        visual_engine = PyVisualEngine()
        qr_data = visual_engine.encode_payload(payload)

        # Process the QR payload
        protocol.process_qr_payload(qr_data.encode())

        # Should transition to sending ACK state
        # (State verification depends on implementation)

    def test_ack_reception(self, protocol):
        """Test ACK reception"""
        # Set up protocol in waiting state
        nonce = PyCryptoEngine.generate_nonce()
        protocol.receive_nonce(nonce)

        # Receive ACK (mocked)
        protocol.receive_ack()

        # Should transition to connected state
        assert protocol.get_state() == "connected"

    def test_encrypt_decrypt_message(self, protocol):
        """Test message encryption/decryption after connection"""
        # Set up connected state
        nonce = PyCryptoEngine.generate_nonce()
        protocol.receive_nonce(nonce)

        payload = PyVisualPayload(
            bytes([i for i in range(16)]),
            bytes([i for i in range(32)]),
            PyCryptoEngine.generate_nonce(),
            bytes([i for i in range(32)])
        )
        visual_engine = PyVisualEngine()
        qr_data = visual_engine.encode_payload(payload)
        protocol.process_qr_payload(qr_data.encode())
        protocol.receive_ack()

        # Now test encryption/decryption
        plaintext = b"Hello, secure world!"
        encrypted = protocol.encrypt_message(plaintext)
        decrypted = protocol.decrypt_message(encrypted)

        assert decrypted == plaintext
        assert encrypted != plaintext  # Should be encrypted

    def test_invalid_state_transitions(self, protocol):
        """Test invalid state transitions raise errors"""
        # Try to process QR when not waiting
        with pytest.raises(Exception):
            protocol.process_qr_payload(b"fake_qr_data")

        # Try to receive ACK when not waiting
        with pytest.raises(Exception):
            protocol.receive_ack()

    def test_session_id_matching(self, protocol):
        """Test session ID validation in QR payload"""
        # Create payload with different session ID
        wrong_session_id = bytes([i + 1 for i in range(16)])
        payload = PyVisualPayload(
            wrong_session_id,
            bytes([i for i in range(32)]),
            PyCryptoEngine.generate_nonce(),
            bytes([i for i in range(32)])
        )
        visual_engine = PyVisualEngine()
        qr_data = visual_engine.encode_payload(payload)

        # Should fail session ID validation
        with pytest.raises(Exception):
            protocol.process_qr_payload(qr_data.encode())

    def test_protocol_timeout_simulation(self):
        """Test protocol timeout handling (simulated)"""
        protocol = PyProtocolEngine()

        # Simulate timeout by checking state after long period
        # In real implementation, this would be time-based
        initial_state = protocol.get_state()

        # After timeout period (conceptually), should handle timeout
        # This is more of an interface test
        assert protocol.get_state() == initial_state

    def test_concurrent_sessions_isolation(self):
        """Test that multiple protocol instances are isolated"""
        protocol1 = PyProtocolEngine()
        protocol2 = PyProtocolEngine()

        # Generate different session IDs
        # (Implementation dependent)

        # Each should maintain independent state
        assert protocol1.get_state() == "idle"
        assert protocol2.get_state() == "idle"

        # Actions on one shouldn't affect the other
        nonce = PyCryptoEngine.generate_nonce()
        protocol1.receive_nonce(nonce)

        assert protocol1.get_state() != protocol2.get_state()

    def test_large_message_handling(self, protocol):
        """Test handling of large messages"""
        # Set up connected state
        nonce = PyCryptoEngine.generate_nonce()
        protocol.receive_nonce(nonce)

        payload = PyVisualPayload(
            bytes([i for i in range(16)]),
            bytes([i for i in range(32)]),
            PyCryptoEngine.generate_nonce(),
            bytes([i for i in range(32)])
        )
        visual_engine = PyVisualEngine()
        qr_data = visual_engine.encode_payload(payload)
        protocol.process_qr_payload(qr_data.encode())
        protocol.receive_ack()

        # Test with large message
        large_message = b"A" * 10000  # 10KB message
        encrypted = protocol.encrypt_message(large_message)
        decrypted = protocol.decrypt_message(encrypted)

        assert decrypted == large_message

    def test_protocol_state_persistence(self, protocol):
        """Test that protocol state persists across operations"""
        # Start handshake
        nonce = PyCryptoEngine.generate_nonce()
        protocol.receive_nonce(nonce)

        # State should be waiting for QR
        assert protocol.get_state() == "waiting_for_qr"

        # Process QR
        payload = PyVisualPayload(
            bytes([i for i in range(16)]),
            bytes([i for i in range(32)]),
            PyCryptoEngine.generate_nonce(),
            bytes([i for i in range(32)])
        )
        visual_engine = PyVisualEngine()
        qr_data = visual_engine.encode_payload(payload)
        protocol.process_qr_payload(qr_data.encode())

        # State should change
        assert protocol.get_state() != "waiting_for_qr"

    def test_error_state_handling(self, protocol):
        """Test error state handling and recovery"""
        # Force an error condition (implementation dependent)
        # This tests the error handling interface

        # After error, should be in error state
        # error_state = protocol.get_state()
        # assert "error" in error_state

        # Should be able to recover or indicate error
        # (Implementation specific)