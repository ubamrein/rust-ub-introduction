// Copyright (c) 2022 Ubique Innovation AG <https://www.ubique.ch>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[derive(Debug, Clone)]
pub enum Atomic {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
    Interrupt
}

#[derive(Debug, Clone)]
pub enum LogicOp {
    And(Expression, Expression),
    Or(Expression, Expression),
}


#[derive(Debug, Clone)]
pub enum Comparison {
    Smaller(Expression, Expression),
    SmallerEquals(Expression, Expression),
    Equals(Expression, Expression),
    Greater(Expression, Expression),
    GreaterEquals(Expression, Expression),
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Plus { left: Expression, right: Expression },
    Minus { left: Expression, right: Expression },
    Mul { left: Expression, right: Expression },
    Div { left: Expression, right: Expression },
    Mod { left: Expression, right: Expression },
    Pow { left: Expression, right: Expression },
    None,
}

/// Expressions
#[derive(Debug, Clone)]
pub struct ConditionalExpression {
    pub condition: Box<Expression>,
    pub body: Vec<Token>,
    pub body_expression: Box<Expression>,
    pub else_body: Vec<Token>,
    pub else_body_expression: Box<Expression>,
}


#[derive(Debug, Clone)]
pub enum Cast {
    String(Expression),
    Int(Expression),
    Bool(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Atomic(Atomic),
    Ident(String),
    LogicOp(Box<LogicOp>),
    Comparison(Box<Comparison>),
    BinaryOp(Box<BinaryOp>),
    Conditional(ConditionalExpression),
    Input(Box<Expression>),
    Cast(Box<Cast>),
}

#[derive(Debug, Clone)]
pub enum Statement {
    VariableAssignment(VariableAssignment),
    Conditional(Conditional),
    Expression(Expression),
    Print(Expression),
    Loop(Loop)
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub condition: Box<Expression>,
    pub body: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct Conditional {
    pub condition: Box<Expression>,
    pub body: Vec<Token>,
    pub else_body: Option<Vec<Token>>,
}

#[derive(Debug, Clone)]
pub struct VariableAssignment {
    pub new_definition: bool,
    pub ident: String,
    pub value: Expression,
}

#[derive(Debug, Clone)]
/// Unsere Sprache besteht aus verschiedenen Tokens
pub enum Token {
    /// Eine Expression ist ein Token, das einen Wert darstellt. Enstprechend kann eine Expression z.B. einer Variable 
    /// hinzugefügt werden.
    Expression(Expression),
    /// Ein Statement führt Code aus, stellt aber keinen Wert dar und kann somit nur alleine stehen
    Statement(Statement),
    /// Vorzeitiges Ende aus einer While-Loop
    Break,
    /// Placeholder für potenzielles early return
    Return
}