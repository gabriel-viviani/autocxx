// Copyright 2021 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

fn main() {
    let path = std::path::PathBuf::from("steam/src");
    let mut b = autocxx_build::Builder::new("src/main.rs", &[&path]).expect_build();
    b.flag_if_supported("-std=c++14")
        .file("steam/src/steam.cc")
        .compile("autocxx-steam-example");
    println!("cargo:rerun-if-changed=src/main.rs");
}
