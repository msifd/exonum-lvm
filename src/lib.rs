// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Cryptocurrency implementation example using [exonum](http://exonum.com/).

#![deny(
    missing_debug_implementations,
    // missing_docs,
    unsafe_code,
    bare_trait_objects
)]

#[macro_use]
extern crate exonum_derive;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate rlua;

pub mod proto;
pub mod currency;
pub mod lvm;