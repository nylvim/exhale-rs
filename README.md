# Rust bindings for `exhale`

High and low level Rust bindings to [exhale](https://gitlab.com/ecodis/exhale) version 1.2.2.

By default, `exhale` is bundled, use `default-features = false` to enable dynamic linking.

When using bundled library, the `low-complexity` feature can be enabled to trade encoding quality for speed.
See [`exhale`'s FAQ](https://gitlab.com/ecodis/exhale/-/wikis/faq#why-is-the-exhale-encoder-so-slow-is-there-a-switch-for-fast-encoding).

## Example

```rust
use exhale::{Encoder, EncoderConfig};

fn main() {
    let config = EncoderConfig::default();
    let mut encoder = Encoder::new(config).unwrap();

    // The input buffer expects interleaved 24-bit PCM data (in i32)
    let input = encoder.input_mut();
    // Fill the input buffer...

    // Encode the frame
    let output = encoder.encode_frame().unwrap();
    println!("Encoded {} bytes", output.len());
}
```
