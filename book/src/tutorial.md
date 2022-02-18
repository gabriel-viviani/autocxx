# Tutorial

Example:


```rust
#[autocxx_build::doctest]
autocxx::include_cpp! {
    #include "url/origin.h"
    generate!("url::Origin")
    safety!(unsafe_ffi)
}

fn main() {
    let o = ffi::url::Origin::CreateFromNormalizedTuple("https",
        "google.com", 443);
    let uri = o.Serialize();
    println!("URI is {}", uri.to_str().unwrap());
}
```
