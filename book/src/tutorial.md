# Tutorial

Example:

```rust
autocxx_integration_tests::doctest!(
    "",
    indoc::indoc!("
        #include <stdint.h>
        inline uint32_t do_math() { return 3; }
    "),
    {
        use autocxx::prelude::*;

        include_cpp! {
            #include "input.h.h"
            generate!("do_math")
            safety!(unsafe_ffi)
        }

        fn main() {
            println!("C++ says the most important numbers is {}.", ffi::do_math());
        }
    }
)
```
