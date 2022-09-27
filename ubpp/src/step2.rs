// Copyright (c) 2022 Ubique Innovation AG <https://www.ubique.ch>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::step1::{
    Atomic, BinaryOp, Cast, Comparison, Conditional, ConditionalExpression, Expression, LogicOp,
    Loop, Statement, Token, VariableAssignment,
};
use pest::{
    iterators::{Pair, Pairs},
    prec_climber::{Assoc, Operator, PrecClimber},
};

use super::*;

pub fn parse_body(body: Pair<Rule>) -> Vec<Token> {
    let mut tokens = vec![];
    for pair in body.into_inner() {
        match pair.as_rule() {
            Rule::break_keyword => tokens.push(Token::Break),
            Rule::statement => {
                let inner = pair.into_inner().next().unwrap();
                let stmt = match inner.as_rule() {
                    Rule::variable_statement => as_var_assignment(inner),
                    Rule::if_statement => as_if_statement(inner),
                    Rule::print_statement => as_print_statement(inner),
                    Rule::expression_statement => Token::Statement(Statement::Expression(
                        as_expression(inner.into_inner().next().unwrap()),
                    )),
                    Rule::while_statement => as_while_statement(inner),
                    _ => continue,
                };
                tokens.push(stmt);
            }
            Rule::expression => {
                tokens.push(Token::Expression(as_expression(pair)));
            }
            _ => unreachable!(),
        }
    }
    tokens
}

pub fn evaluate_num_operations(pair: Pair<Rule>) -> Expression {
    let climber = PrecClimber::new(vec![
        Operator::new(Rule::plus, Assoc::Left) | Operator::new(Rule::minus, Assoc::Left),
        Operator::new(Rule::mul, Assoc::Left)
            | Operator::new(Rule::div, Assoc::Left)
            | Operator::new(Rule::mod_op, Assoc::Left),
        Operator::new(Rule::pow, Assoc::Right),
    ]);
    consume_bin_num_op(pair, &climber)
}

fn consume_bin_num_op(pair: Pair<Rule>, climber: &PrecClimber<Rule>) -> Expression {
    let primary = |pair| consume_bin_num_op(pair, climber);

    let infix = |left: Expression, op: Pair<Rule>, right: Expression| match op.as_rule() {
        Rule::plus => Expression::BinaryOp(Box::new(BinaryOp::Plus { left, right })),
        Rule::minus => Expression::BinaryOp(Box::new(BinaryOp::Minus { left, right })),
        Rule::mod_op => Expression::BinaryOp(Box::new(BinaryOp::Mod { left, right })),
        Rule::mul => Expression::BinaryOp(Box::new(BinaryOp::Mul { left, right })),
        Rule::div => Expression::BinaryOp(Box::new(BinaryOp::Div { left, right })),
        Rule::pow => Expression::BinaryOp(Box::new(BinaryOp::Pow { left, right })),
        p => Expression::Atomic(Atomic::String(format!("{:?}", p))),
    };

    match pair.as_rule() {
        Rule::binary_num_expression => climber.climb(pair.into_inner(), primary, infix),
        Rule::parent_expression => consume_bin_num_op(pair.into_inner().next().unwrap(), climber),
        Rule::rvalue => get_literal(pair.into_inner()),
        p => unreachable!("{:?}", p),
    }
}

fn as_print_statement(inner: Pair<Rule>) -> Token {
    let mut inner = inner.into_inner().skip(1);
    let string = as_expression(inner.next().unwrap());
    Token::Statement(Statement::Print(string))
}

fn as_if_statement(inner: Pair<Rule>) -> Token {
    let mut inner = inner.into_inner().skip(1);
    let c = inner.next().unwrap().into_inner().next().unwrap();
    let condition = as_expression(c);
    let body = parse_body(inner.next().unwrap());
    let mut else_body = None;
    if inner.next().is_some() {
        else_body = Some(parse_body(inner.next().unwrap()));
    }
    Token::Statement(Statement::Conditional(Conditional {
        condition: Box::new(condition),
        body,
        else_body,
    }))
}

fn as_while_statement(inner: Pair<Rule>) -> Token {
    let mut inner = inner.into_inner().skip(1);
    let c = inner.next().unwrap().into_inner().next().unwrap();
    let condition = as_expression(c);
    let body = parse_body(inner.next().unwrap());
    Token::Statement(Statement::Loop(Loop {
        condition: Box::new(condition),
        body,
    }))
}

fn as_var_assignment(pair: Pair<Rule>) -> Token {
    let inner = pair.into_inner().collect::<Vec<_>>();
    let is_new_var = matches!(inner[0].as_rule(), Rule::let_name);
    let ident_name = if is_new_var {
        inner[1].as_str()
    } else {
        inner[0].as_str()
    };
    let expression = if is_new_var {
        inner[2].clone()
    } else {
        inner[1].clone()
    };
    let expression = as_expression(expression);
    let stmt = Statement::VariableAssignment(VariableAssignment {
        new_definition: is_new_var,
        ident: ident_name.to_string(),
        value: expression,
    });
    Token::Statement(stmt)
}

fn as_expression(expression: Pair<Rule>) -> Expression {
    match expression.as_rule() {
        Rule::if_expression => as_if_expression(expression),
        Rule::binary_num_expression => as_binary_number_expression(expression),
        Rule::boolean_operation => as_boolean_operation(expression),
        Rule::boolean_expression => as_boolean_expression(expression),
        Rule::rvalue => as_literal(expression),
        Rule::binary_string_expression => todo! {"Not implemented"},
        Rule::expression => {
            let mut inner = expression.into_inner();
            let expr = inner.next().unwrap();
            if let Some(cast) = inner.next() {
                as_cast(cast, as_expression(expr))
            } else {
                as_expression(expr)
            }
        }
        Rule::input_expression => as_input_expression(expression.into_inner().nth(1).unwrap()),
        _ => unreachable!("{:?}", expression),
    }
}
fn as_cast(cast: Pair<Rule>, expr: Expression) -> Expression {
    let inner = cast
        .into_inner()
        .nth(1)
        .unwrap()
        .into_inner()
        .next()
        .unwrap();
    match inner.as_rule() {
        Rule::string => Expression::Cast(Box::new(Cast::String(expr))),
        Rule::number => Expression::Cast(Box::new(Cast::Int(expr))),
        Rule::bool => Expression::Cast(Box::new(Cast::Bool(expr))),
        _ => unreachable!(),
    }
}

fn as_input_expression(expression: Pair<Rule>) -> Expression {
    Expression::Input(Box::new(as_expression(expression)))
}

fn as_if_expression(inner: Pair<Rule>) -> Expression {
    let mut inner = inner.into_inner().skip(1);
    let condition = as_expression(inner.next().unwrap().into_inner().next().unwrap());
    let body = parse_body(inner.next().unwrap());
    let body_expression = as_expression(inner.next().unwrap().into_inner().next().unwrap());
    inner.next();
    let else_body = parse_body(inner.next().unwrap());
    let else_expression = Box::new(as_expression(
        inner.next().unwrap().into_inner().next().unwrap(),
    ));

    Expression::Conditional(ConditionalExpression {
        condition: Box::new(condition),
        body,
        body_expression: Box::new(body_expression),
        else_body,
        else_body_expression: else_expression,
    })
}

fn as_binary_number_expression(pair: Pair<Rule>) -> Expression {
    evaluate_num_operations(pair)
}

fn as_boolean_operation(pair: Pair<Rule>) -> Expression {
    let mut inner = pair.into_inner();
    let lhs = inner.next().unwrap();
    let op = inner.next().unwrap();
    let rhs = inner.next().unwrap();
    let lhs = match lhs.as_rule() {
        Rule::rvalue => as_literal(lhs),
        Rule::boolean_expression => as_boolean_expression(lhs),
        _ => unreachable!(),
    };
    let rhs = match rhs.as_rule() {
        Rule::rvalue => as_literal(rhs),
        Rule::boolean_expression => as_boolean_expression(rhs),
        _ => unreachable!(),
    };
    match op.as_rule() {
        Rule::and => Expression::LogicOp(Box::new(LogicOp::And(lhs, rhs))),
        Rule::or => Expression::LogicOp(Box::new(LogicOp::Or(lhs, rhs))),
        _ => unreachable!(),
    }
}

fn as_boolean_expression(pair: Pair<Rule>) -> Expression {
    let mut inner = pair.into_inner();
    let lhs = as_literal(inner.next().unwrap());
    let comparison = inner.next().unwrap();
    let rhs = as_literal(inner.next().unwrap());
    as_comparison(lhs, rhs, comparison)
}

fn as_comparison(lhs: Expression, rhs: Expression, pair: Pair<Rule>) -> Expression {
    match pair.into_inner().next().unwrap().as_rule() {
        Rule::smaller_than => Expression::Comparison(Box::new(Comparison::Smaller(lhs, rhs))),
        Rule::smaller_equals => {
            Expression::Comparison(Box::new(Comparison::SmallerEquals(lhs, rhs)))
        }
        Rule::equals => Expression::Comparison(Box::new(Comparison::Equals(lhs, rhs))),
        Rule::greater_equals => {
            Expression::Comparison(Box::new(Comparison::GreaterEquals(lhs, rhs)))
        }
        Rule::greater_than => Expression::Comparison(Box::new(Comparison::Greater(lhs, rhs))),
        _ => unreachable!(),
    }
}

fn as_literal(pair: Pair<Rule>) -> Expression {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::boolean_literal => match inner.into_inner().next().unwrap().as_rule() {
            Rule::true_literal => Expression::Atomic(Atomic::Bool(true)),
            Rule::false_literal => Expression::Atomic(Atomic::Bool(false)),
            _ => unreachable!(),
        },
        Rule::string_literal => {
            let inner = inner.into_inner().nth(1).unwrap().as_str().to_string();
            Expression::Atomic(Atomic::String(inner))
        }
        Rule::variable_name => Expression::Ident(inner.as_str().to_string()),
        Rule::numeric_literal => {
            Expression::Atomic(Atomic::Number(inner.as_str().trim().parse().unwrap()))
        }
        Rule::expression => as_expression(inner),
        p => unreachable!("{:?}", p),
    }
}

fn get_literal(mut pair: Pairs<Rule>) -> Expression {
    let element: Pair<Rule> = pair.next().unwrap();
    match element.as_rule() {
        Rule::numeric_literal => Expression::Atomic(Atomic::Number(
            element.as_str().trim().parse::<f64>().unwrap(),
        )),
        Rule::variable_name => Expression::Ident(element.as_str().to_string()),
        Rule::string_literal => Expression::Atomic(Atomic::String(
            element.into_inner().nth(1).unwrap().as_str().to_string(),
        )),
        p => {
            println!("{:?}", p);
            unreachable!()
        }
    }
}
