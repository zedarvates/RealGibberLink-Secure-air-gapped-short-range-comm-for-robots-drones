use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gibberlink_core::visual::{VisualEngine, VisualPayload};
use gibberlink_core::crypto::CryptoEngine;
use std::sync::Arc;

fn visual_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("visual_operations");

    // QR code encoding benchmark (target: <10ms)
    group.bench_function("qr_encoding", |b| {
        b.iter(|| {
            let visual = VisualEngine::new();
            let crypto = CryptoEngine::new();

            // Create test payload
            let session_id = CryptoEngine::generate_nonce();
            let public_key = crypto.ed25519_public_key().to_vec();
            let nonce = CryptoEngine::generate_nonce();
            let signature = CryptoEngine::generate_nonce().to_vec(); // Mock signature

            let payload = VisualPayload {
                session_id,
                public_key,
                nonce,
                signature,
            };

            let _qr_svg = black_box(visual.encode_payload(&payload));
        });
    });

    // QR code decoding benchmark
    group.bench_function("qr_decoding", |b| {
        b.iter(|| {
            let visual = VisualEngine::new();
            let crypto = CryptoEngine::new();

            // Create and encode payload first
            let session_id = CryptoEngine::generate_nonce();
            let public_key = crypto.ed25519_public_key().to_vec();
            let nonce = CryptoEngine::generate_nonce();
            let signature = CryptoEngine::generate_nonce().to_vec();

            let payload = VisualPayload {
                session_id,
                public_key: public_key.clone(),
                nonce,
                signature: signature.clone(),
            };

            let qr_svg = visual.encode_payload(&payload).unwrap();

            // Simulate QR data extraction (normally from camera)
            // Take first 500 bytes as approximation
            let qr_data = qr_svg.as_bytes()[..qr_svg.as_bytes().len().min(500)].to_vec();

            let _decoded = black_box(visual.decode_payload(&qr_data));
        });
    });

    // Payload creation benchmark
    group.bench_function("payload_creation", |b| {
        b.iter(|| {
            let crypto = CryptoEngine::new();

            let session_id = CryptoEngine::generate_nonce();
            let public_key = crypto.ed25519_public_key().to_vec();
            let nonce = CryptoEngine::generate_nonce();
            let signature = CryptoEngine::generate_nonce().to_vec();

            let _payload = black_box(VisualPayload {
                session_id,
                public_key,
                nonce,
                signature,
            });
        });
    });

    group.finish();
}

fn latency_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("visual_latency");

    // Target: QR display <10ms
    group.bench_function("qr_display_latency", |b| {
        b.iter(|| {
            let visual = VisualEngine::new();
            let crypto = CryptoEngine::new();

            let session_id = CryptoEngine::generate_nonce();
            let public_key = crypto.ed25519_public_key().to_vec();
            let nonce = CryptoEngine::generate_nonce();
            let signature = CryptoEngine::generate_nonce().to_vec();

            let payload = VisualPayload {
                session_id,
                public_key,
                nonce,
                signature,
            };

            let start = std::time::Instant::now();
            let _qr_svg = visual.encode_payload(&payload).unwrap();
            let duration = start.elapsed();

            assert!(duration.as_millis() < 10, "QR display took {}ms", duration.as_millis());
        });
    });

    // QR scanning simulation (decoding latency)
    group.bench_function("qr_scan_latency", |b| {
        b.iter(|| {
            let visual = VisualEngine::new();
            let crypto = CryptoEngine::new();

            // Pre-generate QR data
            let session_id = CryptoEngine::generate_nonce();
            let public_key = crypto.ed25519_public_key().to_vec();
            let nonce = CryptoEngine::generate_nonce();
            let signature = CryptoEngine::generate_nonce().to_vec();

            let payload = VisualPayload {
                session_id,
                public_key: public_key.clone(),
                nonce,
                signature: signature.clone(),
            };

            let qr_svg = visual.encode_payload(&payload).unwrap();
            let qr_data = qr_svg.as_bytes()[..qr_svg.as_bytes().len().min(500)].to_vec();

            let start = std::time::Instant::now();
            let _decoded = visual.decode_payload(&qr_data).unwrap();
            let duration = start.elapsed();

            // Allow more time for decoding (target: <50ms in practice)
            assert!(duration.as_millis() < 100, "QR scan took {}ms", duration.as_millis());
        });
    });

    group.finish();
}

fn throughput_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("visual_throughput");

    // QR codes per second
    group.throughput(criterion::Throughput::Elements(1));
    group.bench_function("qr_generation_throughput", |b| {
        b.iter(|| {
            let visual = VisualEngine::new();
            let crypto = CryptoEngine::new();

            let session_id = CryptoEngine::generate_nonce();
            let public_key = crypto.ed25519_public_key().to_vec();
            let nonce = CryptoEngine::generate_nonce();
            let signature = CryptoEngine::generate_nonce().to_vec();

            let payload = VisualPayload {
                session_id,
                public_key,
                nonce,
                signature,
            };

            let _qr = black_box(visual.encode_payload(&payload).unwrap());
        });
    });

    // Concurrent QR generation
    group.bench_function("concurrent_qr_generation_10", |b| {
        b.iter(|| {
            let visual = Arc::new(VisualEngine::new());

            let handles: Vec<_> = (0..10).map(|_| {
                let visual = Arc::clone(&visual);
                std::thread::spawn(move || {
                    let crypto = CryptoEngine::new();

                    let session_id = CryptoEngine::generate_nonce();
                    let public_key = crypto.ed25519_public_key().to_vec();
                    let nonce = CryptoEngine::generate_nonce();
                    let signature = CryptoEngine::generate_nonce().to_vec();

                    let payload = VisualPayload {
                        session_id,
                        public_key,
                        nonce,
                        signature,
                    };

                    let _qr = visual.encode_payload(&payload).unwrap();
                })
            }).collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

fn payload_size_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("visual_payload_sizes");

    // Benchmark with different key sizes (simulating different crypto params)
    for key_size in [32, 64, 128] {
        group.bench_with_input(format!("payload_size_{}b_keys", key_size), &key_size, |b, size| {
            b.iter(|| {
                let visual = VisualEngine::new();

                // Create payload with specified key size
                let session_id = [0u8; 16];
                let public_key = vec![0u8; *size];
                let nonce = [0u8; 16];
                let signature = vec![0u8; 64];

                let payload = VisualPayload {
                    session_id,
                    public_key,
                    nonce,
                    signature,
                };

                let qr_svg = visual.encode_payload(&payload).unwrap();
                let _size = black_box(qr_svg.len());
            });
        });
    }

    group.finish();
}

fn error_handling_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("visual_error_handling");

    // Invalid QR data handling
    group.bench_function("invalid_qr_handling", |b| {
        b.iter(|| {
            let visual = VisualEngine::new();
            let invalid_data = b"invalid qr data";

            let _result = black_box(visual.decode_payload(invalid_data));
        });
    });

    // Corrupted QR data handling
    group.bench_function("corrupted_qr_handling", |b| {
        b.iter(|| {
            let visual = VisualEngine::new();

            // Create valid QR first, then corrupt it
            let crypto = CryptoEngine::new();
            let session_id = CryptoEngine::generate_nonce();
            let public_key = crypto.ed25519_public_key().to_vec();
            let nonce = CryptoEngine::generate_nonce();
            let signature = CryptoEngine::generate_nonce().to_vec();

            let payload = VisualPayload {
                session_id,
                public_key,
                nonce,
                signature,
            };

            let qr_svg = visual.encode_payload(&payload).unwrap();
            let mut qr_data = qr_svg.as_bytes()[..qr_svg.as_bytes().len().min(500)].to_vec();

            // Corrupt some bytes
            if qr_data.len() > 10 {
                qr_data[5..10].copy_from_slice(&[0, 0, 0, 0, 0]);
            }

            let _result = black_box(visual.decode_payload(&qr_data));
        });
    });

    group.finish();
}

criterion_group!(benches,
    visual_benchmarks,
    latency_benchmarks,
    throughput_benchmarks,
    payload_size_benchmarks,
    error_handling_benchmarks
);
criterion_main!(benches);