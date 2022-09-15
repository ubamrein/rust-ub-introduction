// Copyright (c) 2022 Ubique Innovation AG <https://www.ubique.ch>
// 
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use pest_derive::Parser;

pub mod step1;
pub mod step2;
pub mod step3;
pub mod step4;

#[derive(Parser)]
#[grammar = "/Users/patrickamrein/Documents/Ubique/git/introduction-to-rust/ubpp.pest"]
pub struct UBPP;