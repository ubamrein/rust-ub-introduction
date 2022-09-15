// Copyright (c) 2022 Ubique Innovation AG <https://www.ubique.ch>
// 
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt::Display;

use crate::step1::Atomic;

impl Display for Atomic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Atomic::String(s) => f.write_str(s),
            Atomic::Number(n) => f.write_str(&n.to_string()),
            Atomic::Bool(b) => f.write_str(&b.to_string()),
            Atomic::Null => f.write_str("null"),
            Atomic::Interrupt => f.write_str("<< interrupt >>"),
        }
    }
}