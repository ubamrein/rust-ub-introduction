// Copyright (c) 2022 Ubique Innovation AG <https://www.ubique.ch>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate pest;

use std::collections::HashMap;

use pest::Parser;

use ubpplib::{step2::parse_body, step3::eval_tokens, Rule};

fn main() {
    let input = std::fs::read_to_string("./example.ubpp").unwrap();
    let parse_result = ubpplib::UBPP::parse(Rule::file, &input)
        .unwrap()
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();
    let tokens = parse_body(parse_result);
    let mut global_tokens = HashMap::new();
    let result = eval_tokens(&tokens, &mut global_tokens).unwrap();
    println!("{}", result);
}
