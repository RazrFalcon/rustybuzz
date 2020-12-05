# Benchmarks

## Run

```
cargo +nightly bench
```

## Results

```
test shape_arabic_hb ... bench:     370,792 ns/iter (+/- 5,982)
test shape_arabic_rb ... bench:     681,101 ns/iter (+/- 65,339)
test shape_latin_hb  ... bench:      42,819 ns/iter (+/- 473)
test shape_latin_rb  ... bench:      38,424 ns/iter (+/- 660)
test shape_zalgo_hb  ... bench:      78,971 ns/iter (+/- 926)
test shape_zalgo_rb  ... bench:     210,171 ns/iter (+/- 1,728)
```
