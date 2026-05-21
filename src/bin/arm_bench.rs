use std::time::Instant;

fn main() {
    let data = b"axiom os arm64 blake3 benchmark data";
    let iterations = 10000u64;

    for _ in 0..1000 {
        let _ = blake3::hash(data);
    }

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = blake3::hash(data);
    }
    let elapsed = start.elapsed();

    let ns_per_op = elapsed.as_nanos() as u64 / iterations;
    println!("AXIOM OS Native ARM64 BLAKE3 Benchmark");
    println!("Platform: {}", std::env::consts::ARCH);
    println!("Iterations: {}", iterations);
    println!("ns/op: {}", ns_per_op);
    println!("us/op: {}", elapsed.as_micros() as u64 / iterations);
    let hash = blake3::hash(data);
    println!("Hash: {:02x?}", &hash.as_bytes()[..8]);
}
