# Benchmarks

## Run

```
cargo +nightly bench
```

## Results

```
test shape_arabic_hb ... bench:     388,809 ns/iter (+/- 3,542)
test shape_arabic_rb ... bench:     589,399 ns/iter (+/- 3,816)
test shape_latin_hb  ... bench:      44,504 ns/iter (+/- 391)
test shape_latin_rb  ... bench:      43,612 ns/iter (+/- 490)
test shape_zalgo_hb  ... bench:      81,964 ns/iter (+/- 496)
test shape_zalgo_rb  ... bench:     165,916 ns/iter (+/- 1,483)
```
