const NUM_BLOCKS: usize = 1 << 24;
const NUM_READS: usize = 300_000_000;
const PREFETCH_DISTANCE: usize = 512;

#[derive(Copy, Clone)]
#[repr(align(64))]
struct Block {
    data: [u64; 8],
}

impl Block {
    #[inline(always)]
    fn sum(&self) -> u64 {
        self.data.iter().sum()
    }

    fn ones() -> Self {
        Self { data: [1; 8] }
    }
}

fn main() {
    let blocks = vec![Block::ones(); NUM_BLOCKS];

    let measure = |name, f: fn(&[Block]) -> u64| {
        let mut timings = Vec::new();
        let mut counts = Vec::new();
        let num_iterations = 5;

        for _ in 0..num_iterations {
            let start = std::time::Instant::now();
            let count = f(&blocks);
            counts.push(count);
            timings.push(start.elapsed().as_millis())
        }

        println!(
            "{:<25} took {}/{} ms (min/avg) (count : {})",
            name,
            timings.iter().min().unwrap(),
            timings.iter().sum::<u128>() / num_iterations,
            counts.iter().sum::<u64>() / num_iterations as u64
        );
    };

    measure("sequential", sequential_reads);

    measure("sequential safe", sequential_reads_safe);

    measure("sequential prefetching", sequential_reads_prefetching);

    measure("random", random_reads);

    measure("random safe", random_reads_safe);

    measure("random prefetching", random_reads_prefetching);

    measure("only rng", only_rng);
}

fn sequential_reads(blocks: &[Block]) -> u64 {
    let mut sum = 0;

    for index in 0..NUM_READS {
        let index_in_range = index & (NUM_BLOCKS - 1);
        sum += unsafe { blocks.get_unchecked(index_in_range) }.sum();
    }

    sum
}

fn sequential_reads_safe(blocks: &[Block]) -> u64 {
    blocks.iter().cycle().map(|b| b.sum()).take(NUM_READS).sum()
}

fn sequential_reads_prefetching(blocks: &[Block]) -> u64 {
    let mut sum = 0;

    for index in 0..NUM_READS {
        let prefetch_index_in_range = (index + PREFETCH_DISTANCE) & (NUM_BLOCKS - 1);
        prefetch(blocks, prefetch_index_in_range);

        let index_in_range = index & (NUM_BLOCKS - 1);
        sum += unsafe { blocks.get_unchecked(index_in_range) }.sum();
    }

    sum
}

fn random_reads(blocks: &[Block]) -> u64 {
    let mut rng = fastrand::Rng::new();

    std::iter::repeat_with(|| rng.usize(..NUM_BLOCKS))
        .map(|index| unsafe { blocks.get_unchecked(index) }.sum())
        .take(NUM_READS)
        .sum()
}

fn random_reads_safe(blocks: &[Block]) -> u64 {
    let mut rng = fastrand::Rng::new();

    std::iter::repeat_with(|| rng.usize(..NUM_BLOCKS))
        .map(|index| blocks[index].sum())
        .take(NUM_READS)
        .sum()
}

fn random_reads_prefetching(blocks: &[Block]) -> u64 {
    let mut rng = fastrand::Rng::new();
    let mut prefetched_indices = [42; PREFETCH_DISTANCE];
    let mut sum = 0;

    for i in (0..PREFETCH_DISTANCE).cycle().take(NUM_READS) {
        sum += unsafe { blocks.get_unchecked(prefetched_indices[i]) }.sum();

        let prefetch_index = rng.usize(..NUM_BLOCKS);
        prefetch(blocks, prefetch_index);
        prefetched_indices[i] = prefetch_index;
    }

    sum
}

fn only_rng(_blocks: &[Block]) -> u64 {
    let mut rng = fastrand::Rng::new();

    std::iter::repeat_with(|| rng.usize(..NUM_BLOCKS))
        .take(NUM_READS)
        .sum::<usize>() as u64
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
