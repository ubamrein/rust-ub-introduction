// Copyright (c) 2022 Ubique Innovation AG <https://www.ubique.ch>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{collections::HashMap, io::stdin};

use crate::step1::{
    Atomic, BinaryOp, Comparison, ConditionalExpression, Expression, LogicOp, Loop, Statement,
    Token,
};

impl Expression {
    fn as_atomic(&self, global_scope: &mut HashMap<String, Expression>) -> Result<Atomic, String> {
        eval_expression(self, global_scope)
    }
    fn as_bool(&self, global_scope: &mut HashMap<String, Expression>) -> Result<bool, String> {
        let inner = eval_expression(self, global_scope)?;
        match inner {
            Atomic::String(s) => Ok(s.parse().map_err(|e| format!("{:?}", e))?),
            Atomic::Number(i) => Ok(i == 0.0),
            Atomic::Bool(b) => Ok(b),
            Atomic::Null => Ok(false),
            Atomic::Interrupt => unreachable!(),
        }
    }
    fn as_string(&self, global_scope: &mut HashMap<String, Expression>) -> Result<String, String> {
        let inner = eval_expression(self, global_scope)?;
        match inner {
            Atomic::String(s) => Ok(s),
            Atomic::Number(i) => Ok(i.to_string()),
            Atomic::Bool(b) => Ok(b.to_string()),
            Atomic::Null => Ok("null".to_string()),
            Atomic::Interrupt => unreachable!(),
        }
    }
    fn as_num(&self, global_scope: &mut HashMap<String, Expression>) -> Result<f64, String> {
        let inner = eval_expression(self, global_scope)?;
        match inner {
            Atomic::String(s) => Ok(s.trim().parse().map_err(|e| format!("{:?}", e))?),
            Atomic::Number(i) => Ok(i),
            Atomic::Bool(b) => Ok(b as i32 as f64),
            Atomic::Null => Ok(0.0),
            Atomic::Interrupt => unreachable!(),
        }
    }
}

pub fn eval_tokens(
    tokens: &[Token],
    global_scope: &mut HashMap<String, Expression>,
) -> Result<Atomic, String> {
    let mut last_expression = Atomic::Null;
    for token in tokens {
        match token {
            Token::Expression(e) => {
                last_expression = eval_expression(e, global_scope)?;
                if matches!(last_expression, Atomic::Interrupt) {
                    return Ok(last_expression);
                }
            }
            Token::Statement(stmt) => {
                let is_interrupt = eval_statement(stmt, global_scope)?;
                if matches!(is_interrupt, Atomic::Interrupt) {
                    return Ok(is_interrupt);
                }
            }
            Token::Break => {
                return Ok(Atomic::Interrupt);
            },
            Token::Return => return Ok(Atomic::Interrupt),
        }
    }
    Ok(last_expression)
}

fn eval_statement(
    stmt: &Statement,
    global_scope: &mut HashMap<String, Expression>,
) -> Result<Atomic, String> {
    match stmt {
        Statement::VariableAssignment(assignment) => {
            eval_assignment(assignment, global_scope)?;
            Ok(Atomic::Null)
        }
        Statement::Conditional(c) => eval_conditional(c, global_scope),
        Statement::Expression(e) => {
            let token = eval_expression(e, global_scope)?;
            Ok(token)
        }
        Statement::Print(expression) => {
            let result = eval_expression(expression, global_scope)?;
            // println!("{:?}", result);
            println!("{}", result);
            Ok(result)
        }
        Statement::Loop(loop_statement) => eval_loop(loop_statement, global_scope),
    }
}

fn eval_loop(
    loop_statement: &Loop,
    global_scope: &mut HashMap<String, Expression>,
) -> Result<Atomic, String> {
    let mut condition = loop_statement.condition.as_bool(global_scope)?;
    while condition {
        let token = eval_tokens(&loop_statement.body, global_scope)?;
        if matches!(token, Atomic::Interrupt) {
            return Ok(Atomic::Null);
        }
        condition = loop_statement.condition.as_bool(global_scope)?;
    }
    Ok(Atomic::Null)
}

fn eval_conditional(
    conditional: &crate::step1::Conditional,
    global_scope: &mut HashMap<String, Expression>,
) -> Result<Atomic, String> {
    let condition = conditional.condition.as_bool(global_scope)?;
    if condition {
        let token = eval_tokens(&conditional.body, global_scope)?;
        Ok(token)
    } else if let Some(tokens) = conditional.else_body.as_ref() {
        let token = eval_tokens(tokens, global_scope)?;
        Ok(token)
    } else {
        Ok(Atomic::Null)
    }
}

fn eval_assignment(
    assignment: &crate::step1::VariableAssignment,
    global_scope: &mut HashMap<String, Expression>,
) -> Result<(), String> {
    if assignment.new_definition && global_scope.contains_key(&assignment.ident) {
        return Err(format!("`{}` already defined", assignment.ident));
    } else {
        let result = eval_expression(&assignment.value, global_scope)?;
        global_scope.insert(assignment.ident.to_string(), Expression::Atomic(result));
    }
    Ok(())
}

fn eval_expression(
    e: &Expression,
    global_scope: &mut HashMap<String, Expression>,
) -> Result<Atomic, String> {
    match e {
        Expression::Atomic(atomic) => Ok(atomic.to_owned()),
        Expression::Ident(ident) => {
            let ident_expression = if let Some(expr) = global_scope.get(ident) {
                expr.clone()
            } else {
                return Err(format!("[ERROR] `{}` not defined!", ident));
            };
            eval_expression(&ident_expression, global_scope)
        }
        Expression::Input(expression) => {
            let mut s = String::new();
            println!("{}", expression.as_string(global_scope)?);
            stdin().read_line(&mut s).map_err(|e| format!("{:?}", e))?;
            Ok(Atomic::String(s))
        }
        Expression::LogicOp(logic_operation) => eval_logic_op(logic_operation, global_scope),
        Expression::Comparison(comparison) => eval_comparison(comparison, global_scope),
        Expression::BinaryOp(num_op) => eval_binary_op(num_op, global_scope),
        Expression::Conditional(conditional) => {
            eval_conditional_expression(conditional, global_scope)
        }
        Expression::Cast(cast) => match cast.as_ref() {
            crate::step1::Cast::String(expr) => {
                let result = expr.as_string(global_scope)?;
                Ok(Atomic::String(result))
            }
            crate::step1::Cast::Int(expr) => {
                let result = expr.as_num(global_scope)?;
                Ok(Atomic::Number(result))
            }
            crate::step1::Cast::Bool(expr) => {
                let result = expr.as_bool(global_scope)?;
                Ok(Atomic::Bool(result))
            }
        },
    }
}

fn eval_conditional_expression(
    conditional: &ConditionalExpression,
    global_scope: &mut HashMap<String, Expression>,
) -> Result<Atomic, String> {
    let condition = conditional.condition.as_bool(global_scope)?;
    if condition {
        let _ = eval_tokens(&conditional.body, global_scope);
        eval_expression(&conditional.body_expression, global_scope)
    } else {
        let _ = eval_tokens(&conditional.else_body, global_scope);
        eval_expression(&conditional.else_body_expression, global_scope)
    }
}

fn eval_logic_op(
    logic_operation: &LogicOp,
    global_scope: &mut HashMap<String, Expression>,
) -> Result<Atomic, String> {
    match logic_operation {
        crate::step1::LogicOp::And(lhs, rhs) => {
            let lhs = lhs.as_bool(global_scope)?;
            let rhs = rhs.as_bool(global_scope)?;
            Ok(Atomic::Bool(lhs && rhs))
        }
        crate::step1::LogicOp::Or(lhs, rhs) => {
            let lhs = lhs.as_bool(global_scope)?;
            let rhs = rhs.as_bool(global_scope)?;
            Ok(Atomic::Bool(lhs || rhs))
        }
    }
}

fn eval_comparison(
    comparison: &Comparison,
    global_scope: &mut HashMap<String, Expression>,
) -> Result<Atomic, String> {
    match comparison {
        crate::step1::Comparison::Smaller(lhs, rhs) => {
            if let (Ok(l), Ok(r)) = (lhs.as_num(global_scope), rhs.as_num(global_scope)) {
                Ok(Atomic::Bool(l < r))
            } else if let (Ok(l), Ok(r)) =
                (lhs.as_string(global_scope), rhs.as_string(global_scope))
            {
                Ok(Atomic::Bool(l < r))
            } else {
                Err(format!("Invalid operands to comparison ({:?})", comparison))
            }
        }
        crate::step1::Comparison::SmallerEquals(lhs, rhs) => {
            if let (Ok(l), Ok(r)) = (lhs.as_num(global_scope), rhs.as_num(global_scope)) {
                Ok(Atomic::Bool(l <= r))
            } else if let (Ok(l), Ok(r)) =
                (lhs.as_string(global_scope), rhs.as_string(global_scope))
            {
                Ok(Atomic::Bool(l <= r))
            } else {
                Err("Invalid operands to comparison".to_string())
            }
        }
        crate::step1::Comparison::Equals(lhs, rhs) => {
            if let (Ok(l), Ok(r)) = (lhs.as_num(global_scope), rhs.as_num(global_scope)) {
                Ok(Atomic::Bool(l == r))
            } else if let (Ok(l), Ok(r)) =
                (lhs.as_string(global_scope), rhs.as_string(global_scope))
            {
                Ok(Atomic::Bool(l == r))
            } else {
                Err("Invalid operands to comparison".to_string())
            }
        }
        crate::step1::Comparison::Greater(lhs, rhs) => {
            if let (Ok(l), Ok(r)) = (lhs.as_num(global_scope), rhs.as_num(global_scope)) {
                Ok(Atomic::Bool(l > r))
            } else if let (Ok(l), Ok(r)) =
                (lhs.as_string(global_scope), rhs.as_string(global_scope))
            {
                Ok(Atomic::Bool(l > r))
            } else {
                Err("Invalid operands to comparison".to_string())
            }
        }
        crate::step1::Comparison::GreaterEquals(lhs, rhs) => {
            if let (Ok(l), Ok(r)) = (lhs.as_num(global_scope), rhs.as_num(global_scope)) {
                Ok(Atomic::Bool(l >= r))
            } else if let (Ok(l), Ok(r)) =
                (lhs.as_string(global_scope), rhs.as_string(global_scope))
            {
                Ok(Atomic::Bool(l >= r))
            } else {
                Err("Invalid operands to comparison".to_string())
            }
        }
    }
}

fn eval_binary_op(
    num_op: &BinaryOp,
    global_scope: &mut HashMap<String, Expression>,
) -> Result<Atomic, String> {
    match num_op {
        crate::step1::BinaryOp::Plus { left, right } => {
            let left = left.as_atomic(global_scope)?;
            let right = right.as_atomic(global_scope)?;
            match (&left, &right) {
                (Atomic::String(left), Atomic::String(right)) => {
                    Ok(Atomic::String(left.to_string() + right))
                }
                (Atomic::Number(left), Atomic::String(right)) => {
                    Ok(Atomic::String(left.to_string() + right))
                }
                (Atomic::String(left), Atomic::Number(right)) => {
                    Ok(Atomic::String(left.to_string() + &right.to_string()))
                }
                (Atomic::Number(left), Atomic::Number(right)) => Ok(Atomic::Number(left + right)),
                _ => Err(format!(
                    "Could not evaluate expression {:?} + {:?}",
                    left, right
                )),
            }
        }
        crate::step1::BinaryOp::Minus { left, right } => {
            let left = left.as_num(global_scope)?;
            let right = right.as_num(global_scope)?;
            Ok(Atomic::Number(left - right))
        }
        crate::step1::BinaryOp::Mul { left, right } => {
            let left = left.as_num(global_scope)?;
            let right = right.as_num(global_scope)?;
            Ok(Atomic::Number(left * right))
        }
        crate::step1::BinaryOp::Div { left, right } => {
            let left = left.as_num(global_scope)?;
            let right = right.as_num(global_scope)?;
            Ok(Atomic::Number(left / right))
        }
        crate::step1::BinaryOp::Mod { left, right } => {
            let left = left.as_num(global_scope)?;
            let right = right.as_num(global_scope)?;
            Ok(Atomic::Number(left % right))
        }
        crate::step1::BinaryOp::Pow { left, right } => {
            let left = left.as_num(global_scope)?;
            let right = right.as_num(global_scope)?;
            Ok(Atomic::Number(left.powf(right)))
        }
        crate::step1::BinaryOp::None => unreachable!(),
    }
}
