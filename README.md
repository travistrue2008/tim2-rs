# tim2-rs

An image loader for TIM2 (.tm2) image files

## Usage

Add the crate to your project's Cargo.toml:

```toml
[dependencies]
tim2 = "0.1.0"
```

Here's a basic example of loading the file:

```rust
use tim2;

fn main() {
    let image = tim2::load("./assets/test.tm2");
    println!("image header: {}", image.header());

    /* print the header info for each frame found */
    for (i, frame) in image.get_frame(0) {
        println!("frame[{}]: {}", i, frame.header());
    }
}
```
