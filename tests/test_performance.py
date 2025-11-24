import pytest
import time
import asyncio
import numpy as np
from gibberlink_core import PyCryptoEngine, PyVisualEngine, PyProtocolEngine


class TestPerformance:
    """Performance benchmarking tests for GibberLink protocol"""

    @pytest.fixture
    def benchmark_data(self):
        """Generate test data of various sizes"""
        return {
            'small': b"Hello World",
            'medium': b"A" * 1024,  # 1KB
            'large': b"B" * 10240,  # 10KB
            'xlarge': b"C" * 102400,  # 100KB
        }

    def test_crypto_encrypt_decrypt_performance(self, benchmark, benchmark_data):
        """Benchmark cryptographic operations"""
        crypto = PyCryptoEngine()
        key = crypto.public_key()  # Use public key as test key

        for size_name, data in benchmark_data.items():
            def encrypt_decrypt():
                encrypted = PyCryptoEngine.encrypt_data(key, data)
                decrypted = PyCryptoEngine.decrypt_data(key, encrypted)
                return decrypted

            result = benchmark(encrypt_decrypt)
            assert result == data

    def test_qr_encoding_performance(self, benchmark, benchmark_data):
        """Benchmark QR code encoding performance"""
        visual = PyVisualEngine()

        # Create test payload
        session_id = bytes([i for i in range(16)])
        public_key = bytes([i for i in range(32)])
        nonce = PyCryptoEngine.generate_nonce()
        signature = bytes([i for i in range(32)])

        def encode_qr():
            payload = PyVisualEngine.PyVisualPayload(session_id, public_key, nonce, signature)
            return visual.encode_payload(payload)

        result = benchmark(encode_qr)
        assert isinstance(result, str)
        assert len(result) > 0

    def test_qr_decoding_performance(self, benchmark):
        """Benchmark QR code decoding performance"""
        visual = PyVisualEngine()

        # Create and encode payload
        session_id = bytes([i for i in range(16)])
        public_key = bytes([i for i in range(32)])
        nonce = PyCryptoEngine.generate_nonce()
        signature = bytes([i for i in range(32)])
        payload = PyVisualEngine.PyVisualPayload(session_id, public_key, nonce, signature)
        qr_svg = visual.encode_payload(payload)

        # Simulate QR data extraction (normally from camera)
        qr_data = qr_svg.encode()[:500]  # Truncate for simulation

        def decode_qr():
            return visual.decode_payload(qr_data)

        result = benchmark(decode_qr)
        assert result.session_id == session_id

    def test_protocol_handshake_performance(self, benchmark):
        """Benchmark complete handshake flow"""
        def full_handshake():
            # Device A: Initiate handshake
            device_a = PyProtocolEngine()

            # Generate nonce
            nonce = PyCryptoEngine.generate_nonce()

            # Device B: Receive nonce and generate QR
            device_b = PyProtocolEngine()
            qr_svg = device_b.receive_nonce(nonce)

            # Device A: Process QR payload
            # Simulate QR scanning (normally from camera)
            qr_data = qr_svg.encode()[:500]  # Truncated for simulation

            # Device B: Process QR and send ACK
            device_b.process_qr_payload(qr_data)
            device_b.receive_ack()

            return device_a.get_state() == "connected" and device_b.get_state() == "connected"

        result = benchmark(full_handshake)
        assert result

    @pytest.mark.parametrize("data_size", [100, 1000, 10000])
    def test_message_throughput(self, benchmark, data_size):
        """Test message encryption/decryption throughput"""
        crypto = PyCryptoEngine()
        key = crypto.public_key()
        message = b"A" * data_size

        def encrypt_decrypt_cycle():
            encrypted = PyCryptoEngine.encrypt_data(key, message)
            decrypted = PyCryptoEngine.decrypt_data(key, encrypted)
            return len(decrypted)

        result = benchmark(encrypt_decrypt_cycle)
        assert result == data_size

    def test_latency_measurements(self):
        """Measure latency for various operations"""
        crypto = PyCryptoEngine()

        # Key generation latency
        start = time.perf_counter()
        key = crypto.public_key()
        key_gen_time = time.perf_counter() - start

        # Encryption latency
        data = b"Hello, GibberLink!"
        start = time.perf_counter()
        encrypted = PyCryptoEngine.encrypt_data(key, data)
        encrypt_time = time.perf_counter() - start

        # Decryption latency
        start = time.perf_counter()
        decrypted = PyCryptoEngine.decrypt_data(key, encrypted)
        decrypt_time = time.perf_counter() - start

        # Assert performance targets (burst audio: 20-40ms, QR display: <10ms, total: <300ms)
        assert key_gen_time < 0.01  # <10ms for key generation
        assert encrypt_time < 0.005  # <5ms for encryption
        assert decrypt_time < 0.005  # <5ms for decryption
        assert (key_gen_time + encrypt_time + decrypt_time) < 0.02  # <20ms total

    def test_qr_display_latency(self):
        """Measure QR code generation and display latency"""
        visual = PyVisualEngine()

        session_id = bytes([i for i in range(16)])
        public_key = bytes([i for i in range(32)])
        nonce = PyCryptoEngine.generate_nonce()
        signature = bytes([i for i in range(32)])

        start = time.perf_counter()
        payload = PyVisualEngine.PyVisualPayload(session_id, public_key, nonce, signature)
        qr_svg = visual.encode_payload(payload)
        qr_gen_time = time.perf_counter() - start

        # QR display should be <10ms
        assert qr_gen_time < 0.01
        assert len(qr_svg) > 0

    def test_throughput_under_load(self):
        """Test throughput with multiple concurrent operations"""
        crypto = PyCryptoEngine()
        key = crypto.public_key()
        data = b"Test message for throughput"

        async def single_operation():
            encrypted = PyCryptoEngine.encrypt_data(key, data)
            decrypted = PyCryptoEngine.decrypt_data(key, encrypted)
            return decrypted == data

        async def concurrent_operations():
            tasks = [single_operation() for _ in range(10)]
            results = await asyncio.gather(*tasks)
            return all(results)

        # Run concurrent operations
        import asyncio
        result = asyncio.run(concurrent_operations())
        assert result

    def test_memory_usage_scaling(self):
        """Test memory usage scaling with data size"""
        crypto = PyCryptoEngine()
        key = crypto.public_key()

        sizes = [100, 1000, 10000, 100000]
        for size in sizes:
            data = b"A" * size
            encrypted = PyCryptoEngine.encrypt_data(key, data)
            decrypted = PyCryptoEngine.decrypt_data(key, encrypted)

            # Verify correctness
            assert decrypted == data
            # Memory overhead should be reasonable (nonce + tag overhead)
            assert len(encrypted) <= len(data) + 28  # 12-byte nonce + 16-byte tag

    def test_batch_operation_performance(self, benchmark):
        """Test performance of batch cryptographic operations"""
        crypto = PyCryptoEngine()
        key = crypto.public_key()
        messages = [b"Message " + str(i).encode() for i in range(100)]

        def batch_encrypt_decrypt():
            encrypted_batch = []
            for msg in messages:
                encrypted = PyCryptoEngine.encrypt_data(key, msg)
                encrypted_batch.append(encrypted)

            decrypted_batch = []
            for enc in encrypted_batch:
                decrypted = PyCryptoEngine.decrypt_data(key, enc)
                decrypted_batch.append(decrypted)

            return decrypted_batch

        result = benchmark(batch_encrypt_decrypt)
        assert len(result) == len(messages)
        assert all(orig == dec for orig, dec in zip(messages, result))

    def test_protocol_state_transition_performance(self, benchmark):
        """Benchmark protocol state transitions"""
        def state_transitions():
            protocol = PyProtocolEngine()
            assert protocol.get_state() == "idle"

            # Transition to waiting for QR
            nonce = PyCryptoEngine.generate_nonce()
            qr = protocol.receive_nonce(nonce)
            assert protocol.get_state() == "waiting_for_qr"

            # Transition to sending ACK
            qr_data = qr.encode()[:500]  # Simulate scanned data
            protocol.process_qr_payload(qr_data)
            assert protocol.get_state() == "sending_ack"

            # Transition to connected
            protocol.receive_ack()
            assert protocol.get_state() == "connected"

            return True

        result = benchmark(state_transitions)
        assert result

    @pytest.mark.parametrize("concurrency", [1, 5, 10])
    def test_concurrent_handshakes(self, concurrency):
        """Test multiple concurrent handshake operations"""
        async def single_handshake():
            # Simplified handshake simulation
            crypto = PyCryptoEngine()
            key = crypto.public_key()
            data = b"handshake data"

            encrypted = PyCryptoEngine.encrypt_data(key, data)
            decrypted = PyCryptoEngine.decrypt_data(key, encrypted)

            return decrypted == data

        async def concurrent_handshakes():
            tasks = [single_handshake() for _ in range(concurrency)]
            results = await asyncio.gather(*tasks)
            return all(results)

        import asyncio
        result = asyncio.run(concurrent_handshakes())
        assert result