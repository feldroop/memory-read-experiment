# Memory read experiment

## Results:

- target-cpu: `x86-64` (SSE, SSE2...)

```
sequential                took  1.19 ns/read (single-threaded),  0.89 ns/read (multi-threaded)
sequential safe           took  1.22 ns/read (single-threaded),  0.88 ns/read (multi-threaded)
sequential prefetching    took  1.29 ns/read (single-threaded),  0.75 ns/read (multi-threaded)
random                    took 11.43 ns/read (single-threaded),  1.60 ns/read (multi-threaded)
random safe               took 11.61 ns/read (single-threaded),  1.60 ns/read (multi-threaded)
random prefetching        took  7.63 ns/read (single-threaded),  1.27 ns/read (multi-threaded)
only rng                  took  0.53 ns/read (single-threaded),  0.08 ns/read (multi-threaded)
```

- target-cpu: `x86-64-v2` (SSE3, SSE4...)

```
sequential                took  1.19 ns/read (single-threaded),  0.92 ns/read (multi-threaded)
sequential safe           took  1.19 ns/read (single-threaded),  0.75 ns/read (multi-threaded)
sequential prefetching    took  1.28 ns/read (single-threaded),  0.76 ns/read (multi-threaded)
random                    took 11.42 ns/read (single-threaded),  1.60 ns/read (multi-threaded)
random safe               took 11.55 ns/read (single-threaded),  1.60 ns/read (multi-threaded)
random prefetching        took  7.68 ns/read (single-threaded),  1.27 ns/read (multi-threaded)
only rng                  took  0.56 ns/read (single-threaded),  0.08 ns/read (multi-threaded)
```

- target-cpu: `x86-64-v3` (AVX, AVX2...)

```
sequential                took  1.21 ns/read (single-threaded),  0.84 ns/read (multi-threaded)
sequential safe           took  1.20 ns/read (single-threaded),  0.81 ns/read (multi-threaded)
sequential prefetching    took  1.39 ns/read (single-threaded),  0.56 ns/read (multi-threaded)
random                    took  8.10 ns/read (single-threaded),  1.31 ns/read (multi-threaded)
random safe               took  8.24 ns/read (single-threaded),  1.29 ns/read (multi-threaded)
random prefetching        took  7.32 ns/read (single-threaded),  1.25 ns/read (multi-threaded)
only rng                  took  0.56 ns/read (single-threaded),  0.08 ns/read (multi-threaded)
```

- target-cpu: `x86-64-v4` (AVX512...)

```
sequential                took  1.30 ns/read (single-threaded),  0.91 ns/read (multi-threaded)
sequential safe           took  1.21 ns/read (single-threaded),  0.89 ns/read (multi-threaded)
sequential prefetching    took  1.32 ns/read (single-threaded),  0.62 ns/read (multi-threaded)
random                    took 23.22 ns/read (single-threaded),  3.05 ns/read (multi-threaded)
random safe               took  9.05 ns/read (single-threaded),  1.33 ns/read (multi-threaded)
random prefetching        took  7.06 ns/read (single-threaded),  1.25 ns/read (multi-threaded)
only rng                  took  1.19 ns/read (single-threaded),  0.16 ns/read (multi-threaded)
```

- target-cpu: `native` (AMD Ryzen 7 PRO 8840HS, based on Zen 4)

```
sequential                took  1.31 ns/read (single-threaded),  0.83 ns/read (multi-threaded)
sequential safe           took  5.34 ns/read (single-threaded),  0.49 ns/read (multi-threaded)
sequential prefetching    took  1.64 ns/read (single-threaded),  0.36 ns/read (multi-threaded)
random                    took 25.06 ns/read (single-threaded),  2.09 ns/read (multi-threaded)
random safe               took  9.01 ns/read (single-threaded),  1.40 ns/read (multi-threaded)
random prefetching        took  7.04 ns/read (single-threaded),  1.25 ns/read (multi-threaded)
only rng                  took  0.56 ns/read (single-threaded),  0.08 ns/read (multi-threaded)
```
