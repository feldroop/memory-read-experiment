#![allow(unused)]
use std::{arch::x86_64::_mm_clflush, hint::black_box};

const NUM_BLOCKS: usize = (1 << 24) * 2;
const NUM_READS: usize = 200_000_000;
const PREFETCH_DISTANCE: usize = 32;
const NUM_THREADS: usize = 6;
const STRIDE: usize = 10987654321 % NUM_BLOCKS;

#[derive(Copy, Clone)]
#[repr(align(64))]
struct Block {
    data: [u64; 8],
}

impl Block {
    #[inline(always)]
    fn add_other(&mut self, other: &Block) {
        self.data[0] += other.data[0];
        // for (this, other) in self.data.iter_mut().zip(&other.data) {
        // *this += other;
        // }
    }

    #[inline(always)]
    fn accumulate_sum(&self) -> u64 {
        self.data[0]
        // self.data.iter().sum()
    }

    fn zeros() -> Self {
        Self { data: [0; 8] }
    }

    fn ones() -> Self {
        Self { data: [1; 8] }
    }

    fn flush(&self) {
        // unsafe { _mm_clflush(self as *const Block as *const u8) };
    }
}

fn main() {
    let blocks = vec![Block::ones(); NUM_BLOCKS * 2 * NUM_THREADS];
    let blocks = blocks.chunks(NUM_BLOCKS).collect::<Vec<&[Block]>>();

    let flush_all = || {
        for &blocks in &blocks {
            for block in blocks {
                block.flush();
            }
        }
    };

    eprintln!(
        "{:<25} | {:>3} t | {:>3} t | {:>3} t |",
        "Method         (ns/read)",
        1,
        NUM_THREADS,
        2 * NUM_THREADS
    );

    let measure = |name, f: fn(&[Block]) -> u64| {
        let mut single_threaded_timings: Vec<u128> = Vec::new();
        let mut multi_threaded_timings: Vec<u128> = Vec::new();
        let mut hyper_threaded_timings: Vec<u128> = Vec::new();
        let num_iterations = 1;

        for _ in 0..num_iterations {
            flush_all();
            let start = std::time::Instant::now();

            let count = f(&blocks[0]);
            black_box(count);
            // if count != NUM_READS as u64 && name != "only rng" {
            //     panic!("Wrong count");
            // }

            single_threaded_timings.push(start.elapsed().as_nanos());

            flush_all();
            let start = std::time::Instant::now();

            let blocks = &blocks;
            std::thread::scope(|s| {
                let mut handles = Vec::new();
                for t in 0..NUM_THREADS {
                    let handle = s.spawn(move || {
                        let mut count = f(&blocks[t]);
                        // eprintln!("{t}: mid   {}", start.elapsed().as_secs_f64());
                        // count += f(&blocks[t]);
                        // eprintln!("{t}: END   {}", start.elapsed().as_secs_f64());
                        count
                    });
                    handles.push(handle);
                }

                // eprintln!("joining..");

                for handle in handles {
                    let count = handle.join().unwrap();
                    black_box(count);

                    // if count != NUM_READS as u64 && name != "only rng" {
                    //     panic!("Wrong count");
                    // }
                }
            });

            multi_threaded_timings.push(start.elapsed().as_nanos());

            flush_all();
            let start = std::time::Instant::now();

            std::thread::scope(|s| {
                let mut handles = Vec::new();
                for t in 0..2 * NUM_THREADS {
                    let handle = s.spawn(move || f(&blocks[t]));
                    handles.push(handle);
                }

                for handle in handles {
                    let count = handle.join().unwrap();
                    black_box(count);

                    // if count != NUM_READS as u64 * 8 && name != "only rng" {
                    //     panic!("Wrong count");
                    // }
                }
            });

            hyper_threaded_timings.push(start.elapsed().as_nanos());
        }

        let min_inverse_throughput_single = single_threaded_timings
            .iter()
            .map(|&t| t as f64 / NUM_READS as f64 / 1.0)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let min_inverse_throughput_multi = multi_threaded_timings
            .iter()
            .map(|&t| t as f64 / NUM_READS as f64 / 1.0)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let min_inverse_throughput_hyper = hyper_threaded_timings
            .iter()
            .map(|&t| t as f64 / NUM_READS as f64 / 1.0)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        eprintln!(
            "{:<25} | {:>5.3} | {:>5.3} | {:>5.3} |",
            name,
            min_inverse_throughput_single,
            min_inverse_throughput_multi / NUM_THREADS as f64,
            min_inverse_throughput_hyper / (2 * NUM_THREADS) as f64,
        );
    };

    measure("sequential", sequential_reads);
    // measure("sequential safe", sequential_reads_safe);
    measure("sequential pf", sequential_reads_prefetching);

    measure("sequential offset", sequential_reads_offset);
    // measure("sequential safe", sequential_reads_safe);
    measure("sequential offset pf", sequential_reads_offset_prefetching);

    measure("dbl sequential", double_sequential_reads::<2>);
    // measure("dbl sequential safe", double_sequential_reads_safe::<2>);
    measure(
        "dbl sequential pf",
        double_sequential_reads_prefetching::<2>,
    );

    measure("trip sequential", double_sequential_reads::<3>);
    measure(
        "trip sequential pf",
        double_sequential_reads_prefetching::<3>,
    );

    measure("quad sequential", double_sequential_reads::<4>);
    measure(
        "quad sequential pf",
        double_sequential_reads_prefetching::<4>,
    );

    measure("oct sequential", double_sequential_reads::<8>);
    measure(
        "oct sequential pf",
        double_sequential_reads_prefetching::<8>,
    );

    measure("hex sequential", double_sequential_reads::<16>);
    measure(
        "hex sequential pf",
        double_sequential_reads_prefetching::<16>,
    );

    measure("32 sequential", double_sequential_reads::<32>);
    measure(
        "32 sequential pf",
        double_sequential_reads_prefetching::<32>,
    );

    measure("64 sequential", double_sequential_reads::<64>);
    measure(
        "64 sequential pf",
        double_sequential_reads_prefetching::<64>,
    );

    measure("random", random_reads);
    measure("random safe", random_reads_safe);
    measure("random pf", random_reads_prefetching);
    // measure("only rng", only_rng);

    measure("stride", stride_reads);
    measure("stride safe", stride_reads_safe);
    measure("stride pf", stride_reads_prefetching);
}

#[inline(never)]
fn sequential_reads(blocks: &[Block]) -> u64 {
    let mut accumulator_block = Block::zeros();

    for index in 0..NUM_READS {
        let index_in_range = index & (NUM_BLOCKS - 1);
        accumulator_block.add_other(unsafe { blocks.get_unchecked(index_in_range) });
    }

    accumulator_block.accumulate_sum()
}

#[inline(never)]
fn sequential_reads_safe(blocks: &[Block]) -> u64 {
    blocks
        .iter()
        .cycle()
        .take(NUM_READS)
        .fold(Block::zeros(), |mut accumulator_block: Block, block| {
            accumulator_block.add_other(block);
            accumulator_block
        })
        .accumulate_sum()
}

#[inline(never)]
fn sequential_reads_prefetching(blocks: &[Block]) -> u64 {
    let mut accumulator_block = Block::zeros();

    for index in 0..NUM_READS {
        let prefetch_index_in_range = (index + PREFETCH_DISTANCE) & (NUM_BLOCKS - 1);
        prefetch(blocks, prefetch_index_in_range);

        let index_in_range = index & (NUM_BLOCKS - 1);
        accumulator_block.add_other(unsafe { blocks.get_unchecked(index_in_range) });
    }

    accumulator_block.accumulate_sum()
}

#[inline(never)]
fn sequential_reads_offset(blocks: &[Block]) -> u64 {
    let mut accumulator_block = Block::zeros();
    let mut offset = fastrand::Rng::new().usize(0..NUM_BLOCKS);

    for index in 0..NUM_READS {
        let index_in_range = (offset + index) & (NUM_BLOCKS - 1);
        accumulator_block.add_other(unsafe { blocks.get_unchecked(index_in_range) });
    }

    accumulator_block.accumulate_sum()
}

#[inline(never)]
fn sequential_reads_offset_prefetching(blocks: &[Block]) -> u64 {
    let mut accumulator_block = Block::zeros();
    let mut offset = fastrand::Rng::new().usize(0..NUM_BLOCKS);

    for index in 0..NUM_READS {
        let prefetch_index_in_range = (index + offset + PREFETCH_DISTANCE) & (NUM_BLOCKS - 1);
        prefetch(blocks, prefetch_index_in_range);

        let index_in_range = (index + offset) & (NUM_BLOCKS - 1);
        accumulator_block.add_other(unsafe { blocks.get_unchecked(index_in_range) });
    }

    accumulator_block.accumulate_sum()
}

#[inline(never)]
fn double_sequential_reads<const B: usize>(blocks: &[Block]) -> u64 {
    let mut accumulator_block = Block::zeros();

    for index in 0..NUM_READS {
        let index_in_range = (B * index) & (NUM_BLOCKS - 1);
        accumulator_block.add_other(unsafe { blocks.get_unchecked(index_in_range) });
    }

    accumulator_block.accumulate_sum()
}

#[inline(never)]
fn double_sequential_reads_safe<const B: usize>(blocks: &[Block]) -> u64 {
    blocks
        .iter()
        .step_by(B)
        .cycle()
        .take(NUM_READS)
        .fold(Block::zeros(), |mut accumulator_block: Block, block| {
            accumulator_block.add_other(block);
            accumulator_block
        })
        .accumulate_sum()
}

#[inline(never)]
fn double_sequential_reads_prefetching<const B: usize>(blocks: &[Block]) -> u64 {
    let mut accumulator_block = Block::zeros();

    for index in 0..NUM_READS {
        let prefetch_index_in_range = (B * (index + PREFETCH_DISTANCE)) & (NUM_BLOCKS - 1);
        prefetch(blocks, prefetch_index_in_range);

        let index_in_range = (B * index) & (NUM_BLOCKS - 1);
        accumulator_block.add_other(unsafe { blocks.get_unchecked(index_in_range) });
    }

    accumulator_block.accumulate_sum()
}

#[inline(never)]
fn random_reads(blocks: &[Block]) -> u64 {
    let mut rng = fastrand::Rng::new();

    std::iter::repeat_with(|| rng.usize(..NUM_BLOCKS))
        .take(NUM_READS)
        .fold(Block::zeros(), |mut accumulator_block: Block, index| {
            accumulator_block.add_other(unsafe { blocks.get_unchecked(index) });
            accumulator_block
        })
        .accumulate_sum()
}

#[inline(never)]
fn random_reads_safe(blocks: &[Block]) -> u64 {
    let mut rng = fastrand::Rng::new();

    std::iter::repeat_with(|| rng.usize(..NUM_BLOCKS))
        .take(NUM_READS)
        .fold(Block::zeros(), |mut accumulator_block: Block, index| {
            accumulator_block.add_other(&blocks[index]);
            accumulator_block
        })
        .accumulate_sum()
}

#[inline(never)]
fn random_reads_prefetching(blocks: &[Block]) -> u64 {
    let mut rng = fastrand::Rng::new();
    let mut prefetched_indices = [42; PREFETCH_DISTANCE];
    let mut accumulator_block = Block::zeros();

    for i in (0..PREFETCH_DISTANCE).cycle().take(NUM_READS) {
        accumulator_block.add_other(unsafe { blocks.get_unchecked(prefetched_indices[i]) });
        let prefetch_index = rng.usize(..NUM_BLOCKS);
        prefetch(blocks, prefetch_index);
        prefetched_indices[i] = prefetch_index;
    }

    accumulator_block.accumulate_sum()
}

#[inline(never)]
fn only_rng(_blocks: &[Block]) -> u64 {
    let mut rng = fastrand::Rng::new();

    std::iter::repeat_with(|| rng.usize(..NUM_BLOCKS))
        .take(NUM_READS)
        .sum::<usize>() as u64
}

#[inline(never)]
fn stride_reads(blocks: &[Block]) -> u64 {
    let mut pos = 0;
    std::iter::repeat_with(|| {
        pos = (pos + STRIDE) % NUM_BLOCKS;
        pos
    })
    .take(NUM_READS)
    .fold(Block::zeros(), |mut accumulator_block: Block, index| {
        accumulator_block.add_other(unsafe { blocks.get_unchecked(index) });
        accumulator_block
    })
    .accumulate_sum()
}

#[inline(never)]
fn stride_reads_safe(blocks: &[Block]) -> u64 {
    let mut pos = 0;

    std::iter::repeat_with(|| {
        pos = (pos + STRIDE) % NUM_BLOCKS;
        pos
    })
    .take(NUM_READS)
    .fold(Block::zeros(), |mut accumulator_block: Block, index| {
        accumulator_block.add_other(&blocks[index]);
        accumulator_block
    })
    .accumulate_sum()
}

#[inline(never)]
fn stride_reads_prefetching(blocks: &[Block]) -> u64 {
    let mut pos = 0;
    let mut prefetched_indices = [42; PREFETCH_DISTANCE];
    let mut accumulator_block = Block::zeros();

    for i in (0..PREFETCH_DISTANCE).cycle().take(NUM_READS) {
        accumulator_block.add_other(unsafe { blocks.get_unchecked(prefetched_indices[i]) });
        pos = (pos + STRIDE) % NUM_BLOCKS;
        prefetch(blocks, pos);
        prefetched_indices[i] = pos;
    }

    accumulator_block.accumulate_sum()
}

#[inline(always)]
fn prefetch<T>(data: impl AsRef<[T]>, index: usize) {
    let ptr = data.as_ref().as_ptr().wrapping_add(index) as *const i8;

    #[cfg(target_arch = "x86_64")]
    unsafe {
        std::arch::x86_64::_mm_prefetch(ptr, std::arch::x86_64::_MM_HINT_T0);
    }

    #[cfg(target_arch = "x86")]
    unsafe {
        std::arch::x86::_mm_prefetch(ptr, std::arch::x86::_MM_HINT_T0);
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        std::arch::aarch64::_prefetch(ptr, std::arch::aarch64::_PREFETCH_LOCALITY3);
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64")))]
    {
        // Do nothing.
    }
}
