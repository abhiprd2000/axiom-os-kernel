use std::hint::black_box;
use std::time::Instant;

const BLOCK: usize = 4096;

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n == 0 {
        return 0.0;
    }
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

fn bench<F: FnMut()>(trials: usize, iters: u64, mut f: F) -> f64 {
    for _ in 0..iters.min(64) {
        f();
    }
    let mut per_op = Vec::with_capacity(trials);
    for _ in 0..trials {
        let start = Instant::now();
        for _ in 0..iters {
            f();
        }
        per_op.push(start.elapsed().as_nanos() as f64 / iters as f64);
    }
    median(per_op)
}

fn inline_vs_periodic() {
    const NBLOCKS: usize = 256;
    const OPS: usize = 200_000;
    const VICTIM: usize = 123;
    const TAMPER_AT: usize = OPS / 2;

    let mut data = vec![0u8; NBLOCKS * BLOCK];
    for (i, b) in data.iter_mut().enumerate() {
        *b = (i as u32).wrapping_mul(2654435761) as u8;
    }
    let leaves: Vec<[u8; 32]> = (0..NBLOCKS)
        .map(|i| *blake3::hash(&data[i * BLOCK..(i + 1) * BLOCK]).as_bytes())
        .collect();

    let verify = |blk: &[u8], stored: &[u8; 32]| -> bool {
        let h = blake3::hash(blk);
        let a = h.as_bytes();
        let mut d = 0u8;
        for i in 0..32 {
            d |= a[i] ^ stored[i];
        }
        d == 0
    };

    let mut s: u64 = 0x9E3779B97F4A7C15;
    let mut rng = || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s
    };
    let trace: Vec<usize> = (0..OPS).map(|_| (rng() as usize) % NBLOCKS).collect();

    let blk0 = &data[VICTIM * BLOCK..(VICTIM + 1) * BLOCK];
    let inline_ns = bench(7, 4000, || {
        black_box(verify(black_box(blk0), &leaves[VICTIM]));
    });
    let scan_ns = bench(7, 200, || {
        let mut bad = false;
        for i in 0..NBLOCKS {
            bad |= !verify(&data[i * BLOCK..(i + 1) * BLOCK], &leaves[i]);
        }
        black_box(bad);
    });

    let inline_detect = trace[TAMPER_AT..].iter().position(|&b| b == VICTIM);

    println!();
    println!(
        "--- inline per-read vs periodic scan  ({} blocks = {} KiB, {} ops) ---",
        NBLOCKS,
        NBLOCKS * BLOCK / 1024,
        OPS
    );
    println!(
        "inline per-read : {:8.4} us/op | corrupted reads served = 0 | detection = next read of a block",
        inline_ns / 1000.0
    );
    if let Some(d) = inline_detect {
        println!("                  (victim first re-read {} ops after tamper; caught there, 0 corrupt bytes served)", d);
    }
    println!(
        "{:>14}  {:>12}  {:>22}  {:>16}",
        "scan_interval", "amort_us/op", "corrupted_reads_served", "detect_window_ops"
    );
    for &iv in &[1000usize, 10_000, 50_000, 100_000] {
        let mut next_scan = TAMPER_AT;
        while next_scan % iv != iv - 1 {
            next_scan += 1;
        }
        let next_scan = next_scan.min(OPS - 1);
        let corrupted = trace[TAMPER_AT..=next_scan]
            .iter()
            .filter(|&&b| b == VICTIM)
            .count();
        let amort = (scan_ns / iv as f64) / 1000.0;
        println!(
            "{:>14}  {:>12.4}  {:>22}  {:>16}",
            iv,
            amort,
            corrupted,
            next_scan - TAMPER_AT
        );
    }
    println!("tradeoff: inline pays per read but serves 0 corrupted bytes; periodic is cheaper");
    println!(
        "          per op as the interval grows, but serves corrupted data across the window."
    );
}

fn main() {
    let arch = std::env::consts::ARCH;
    println!("axiom-bench-native   arch={arch}   (BLAKE3 std, runtime SIMD)");
    println!("metric: median ns/op over 7 trials; warm-up applied");
    println!();

    println!("--- BLAKE3 throughput ---");
    for &sz in &[64usize, 512, 4096] {
        let buf = vec![0xABu8; sz];
        let ns = bench(7, 5000, || {
            let h = blake3::hash(black_box(&buf));
            black_box(h);
        });
        let gbs = sz as f64 / ns;
        println!("  hash {sz:>6} B: {ns:8.1} ns/op   ({gbs:5.2} GB/s)");
    }
    println!();

    println!("--- read+verify: whole-file vs one 4 KiB block ---");
    println!(
        "{:>10}  {:>7}  {:>14}  {:>12}  {:>8}",
        "size(B)", "blocks", "wholefile(ms)", "block(ms)", "speedup"
    );
    for &sz in &[4096usize, 16384, 65536, 262144, 1048576, 4194304] {
        let buf = vec![0xABu8; sz];
        let nblocks = (sz + BLOCK - 1) / BLOCK;
        let first = &buf[..BLOCK.min(sz)];
        let iters = (8_000_000u64 / sz as u64).max(50);
        let wf = bench(7, iters, || {
            let h = blake3::hash(black_box(&buf));
            black_box(h);
        });
        let bl = bench(7, iters.max(4000), || {
            let h = blake3::hash(black_box(first));
            black_box(h);
        });
        println!(
            "{:>10}  {:>7}  {:>14.6}  {:>12.6}  {:>7.1}x",
            sz,
            nblocks,
            wf / 1e6,
            bl / 1e6,
            wf / bl
        );
    }
    println!();
    println!("speedup = proportional-access effect (platform-independent in shape).");
    println!("absolute ms is specific to this CPU; label it arch={arch} in the paper.");
    inline_vs_periodic();
}
