cat > src/main.rs << 'EOF'
use std::hint::black_box;
use std::time::Instant;

const BLOCK: usize = 4096;

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n == 0 { return 0.0; }
    if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2.0 }
}

fn bench<F: FnMut()>(trials: usize, iters: u64, mut f: F) -> f64 {
    for _ in 0..iters.min(64) { f(); }
    let mut per_op = Vec::with_capacity(trials);
    for _ in 0..trials {
        let start = Instant::now();
        for _ in 0..iters { f(); }
        per_op.push(start.elapsed().as_nanos() as f64 / iters as f64);
    }
    median(per_op)
}

fn main() {
    let arch = std::env::consts::ARCH;
    println!("axiom-bench-native   arch={arch}   (BLAKE3 std, runtime SIMD)");
    println!("metric: median ns/op over 7 trials; warm-up applied");
    println!();

    println!("--- BLAKE3 throughput ---");
    for &sz in &[64usize, 512, 4096] {
        let buf = vec![0xABu8; sz];
        let ns = bench(7, 5000, || { let h = blake3::hash(black_box(&buf)); black_box(h); });
        let gbs = sz as f64 / ns;
        println!("  hash {sz:>6} B: {ns:8.1} ns/op   ({gbs:5.2} GB/s)");
    }
    println!();

    println!("--- read+verify: whole-file vs one 4 KiB block ---");
    println!("{:>10}  {:>7}  {:>14}  {:>12}  {:>8}", "size(B)", "blocks", "wholefile(ms)", "block(ms)", "speedup");
    for &sz in &[4096usize, 16384, 65536, 262144, 1048576, 4194304] {
        let buf = vec![0xABu8; sz];
        let nblocks = (sz + BLOCK - 1) / BLOCK;
        let first = &buf[..BLOCK.min(sz)];
        let iters = (8_000_000u64 / sz as u64).max(50);
        let wf = bench(7, iters, || { let h = blake3::hash(black_box(&buf)); black_box(h); });
        let bl = bench(7, iters.max(4000), || { let h = blake3::hash(black_box(first)); black_box(h); });
        println!("{:>10}  {:>7}  {:>14.6}  {:>12.6}  {:>7.1}x", sz, nblocks, wf / 1e6, bl / 1e6, wf / bl);
    }
    println!();
    println!("speedup = proportional-access effect (platform-independent in shape).");
    println!("absolute ms is specific to this CPU; label it arch={arch} in the paper.");
}
EOF