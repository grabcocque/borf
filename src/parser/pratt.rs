use crate::parser::ast::{Expression, Identifier, TypeExpr};
use crate::parser::error::BorfError;
use crate::parser::Rule;
use pest::iterators::{Pair, Pairs};
use std::iter::Peekable;

// --- Operator Definition ---

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operator {
    // Infix
    // Assign,            // = (Though maybe handled differently)
    Or,                // $or
    And,               // $and
    Iff,               // $iff
    Implies,           // =>
    TransitiveClosure, // ->+
    Eq,                // Equality test function (language primitive)
    // NotEqual,          // (Implicitly handled via $not?) - Need to check grammar/AST
    LessThan,       // <
    LessEqual,      // <=
    GreaterThan,    // >
    GreaterEqual,   // >=
    CustomEquality, // $teq, $veq, $seq, $ceq
    Membership,     // $in
    Omega,          // $omega (Type relation?)
    TypeConstraint, // <::
    HasType,        // : (Type annotation in expressions?)
    SetUnion,       // $cup
    SetIntersect,   // $cap
    SetSubset,      // $subseteq
    Add,            // +
    Subtract,       // -
    Multiply,       // *
    Divide,         // /
    Compose,        // . , >>, |>
    // Prefix
    Not,             // $not
    CardinalityOpen, // | (Prefix usage)
    // Postfix
    CardinalityClose, // | (Postfix usage)
    // Ternary - Special Handling Needed
    TernaryQ, // ?
    TernaryC, // : (Ternary context)
    // Others
    FunctionCall, // Implicit operator for f(x)
    FieldAccess,  // Implicit operator for a.b
    IndexAccess,  // Implicit operator for a[b]
    // Add fallible operators
    FallibleSeq,       // $seq
    FallibleAlt,       // $alt
    FallibleNarrow,    // $narrow
    FallibleMap,       // $map
    FallibleDefault,   // $default
    FallibleChoice,    // $choice
    FallibleConstrain, // $constrain
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Lowest,
    Assignment,        // =
    Ternary,           // ? :
    Or,                // $or
    And,               // $and
    Iff,               // $iff
    Implies,           // =>
    Equality,          // ==, !=, ===, $teq, etc. <::, : (has type)
    Comparison,        // <, >, <=, >=, $in, $omega
    Set,               // $cup, $cap, $subseteq
    Term,              // +, -
    Factor,            // *, /
    Arrow,             // ->, ->+, :-> (type operators)
    TransitiveClosure, // ->+ (Higher than Factor?)
    Compose,           // ., >>, |>
    Prefix,            // -, $not, | (prefix)
    Postfix,           // Function call, field access, index access, | (postfix)
    Primary,           // Literals, identifiers, grouping
    Fallible,          // $seq, $alt, $narrow, etc.
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Associativity {
    Left,
    Right,
    None,
}

impl Operator {
    fn from_pair(pair: &Pair<Rule>) -> Option<Operator> {
        match pair.as_rule() {
            // Infix
            Rule::op_logical_or => Some(Operator::Or),
            Rule::op_logical_and => Some(Operator::And),
            Rule::op_iff => Some(Operator::Iff),
            Rule::op_implication => Some(Operator::Implies),
            Rule::op_comparison => {
                match pair.as_str() {
                    "$veq" | "$teq" | "$seq" | "$ceq" => Some(Operator::CustomEquality),
                    "<::" => Some(Operator::TypeConstraint),
                    ">" => Some(Operator::GreaterThan),
                    "<" => Some(Operator::LessThan),
                    ">=" => Some(Operator::GreaterEqual),
                    "<=" => Some(Operator::LessEqual),
                    "$in" => Some(Operator::Membership),
                    "$omega" => Some(Operator::Omega),
                    ":" => Some(Operator::HasType), // Or could be TernaryC
                    _ => None,                      // Should not happen based on grammar rule
                }
            }
            Rule::op_set => match pair.as_str() {
                "$cup" => Some(Operator::SetUnion),
                "$cap" => Some(Operator::SetIntersect),
                "$subseteq" => Some(Operator::SetSubset),
                _ => None,
            },
            Rule::op_additive => match pair.as_str() {
                "+" => Some(Operator::Add),
                "-" => Some(Operator::Subtract),
                _ => None,
            },
            Rule::op_multiplicative => match pair.as_str() {
                "*" => Some(Operator::Multiply),
                "/" => Some(Operator::Divide),
                _ => None,
            },
            Rule::op_transitive_closure => Some(Operator::TransitiveClosure),
            Rule::op_composition => Some(Operator::Compose), // Handles ., >>, |>
            // Prefix
            Rule::op_prefix_not => Some(Operator::Not),
            Rule::op_prefix_cardinality_open => Some(Operator::CardinalityOpen),
            // Postfix
            Rule::op_postfix_cardinality_close => Some(Operator::CardinalityClose),
            // Ternary
            Rule::op_ternary_q => Some(Operator::TernaryQ),
            Rule::op_ternary_c => Some(Operator::TernaryC), // Ambiguous with HasType, context needed
            // Add fallible operators
            Rule::op_fallible => match pair.as_str() {
                "$seq" => Some(Operator::FallibleSeq),
                "$alt" => Some(Operator::FallibleAlt),
                "$narrow" => Some(Operator::FallibleNarrow),
                "$map" => Some(Operator::FallibleMap),
                "$default" => Some(Operator::FallibleDefault),
                "$choice" => Some(Operator::FallibleChoice),
                "$constrain" => Some(Operator::FallibleConstrain),
                _ => None,
            },
            _ => None,
        }
    }

    fn precedence(&self) -> Precedence {
        match self {
            // Operator::Assign => Precedence::Assignment,
            Operator::TernaryQ | Operator::TernaryC => Precedence::Ternary,
            Operator::Or => Precedence::Or,
            Operator::And => Precedence::And,
            Operator::Iff => Precedence::Iff,
            Operator::Implies => Precedence::Implies,
            Operator::Eq
            // | Operator::NotEqual
            | Operator::CustomEquality
            | Operator::TypeConstraint
            | Operator::HasType => Precedence::Equality,
            Operator::LessThan
            | Operator::LessEqual
            | Operator::GreaterThan
            | Operator::GreaterEqual
            | Operator::Membership
            | Operator::Omega => Precedence::Comparison,
            Operator::SetUnion | Operator::SetIntersect | Operator::SetSubset => Precedence::Set,
            Operator::Add | Operator::Subtract => Precedence::Term,
            Operator::Multiply | Operator::Divide => Precedence::Factor,
            Operator::TransitiveClosure => Precedence::TransitiveClosure,
            Operator::Compose => Precedence::Compose,
            Operator::Not | Operator::CardinalityOpen => Precedence::Prefix,
            Operator::FunctionCall
            | Operator::FieldAccess
            | Operator::IndexAccess
            | Operator::CardinalityClose => Precedence::Postfix,
            Operator::FallibleSeq | Operator::FallibleAlt | Operator::FallibleNarrow |
            Operator::FallibleMap | Operator::FallibleDefault | Operator::FallibleChoice |
            Operator::FallibleConstrain => Precedence::Fallible,
        }
    }

    fn associativity(&self) -> Associativity {
        match self {
            // Most binary operators are left-associative
            Operator::Or
            | Operator::And
            | Operator::Eq
            // | Operator::NotEqual
            | Operator::CustomEquality
            | Operator::LessThan
            | Operator::LessEqual
            | Operator::GreaterThan
            | Operator::GreaterEqual
            | Operator::Membership
            | Operator::Omega
            | Operator::SetUnion
            | Operator::SetIntersect
            | Operator::SetSubset
            | Operator::Add
            | Operator::Subtract
            | Operator::Multiply
            | Operator::Divide
            | Operator::Compose
            | Operator::TransitiveClosure
            | Operator::FunctionCall
            | Operator::FieldAccess
            | Operator::IndexAccess
            | Operator::FallibleSeq
            | Operator::FallibleAlt
            | Operator::FallibleNarrow
            | Operator::FallibleMap
            | Operator::FallibleDefault
            | Operator::FallibleChoice
            | Operator::FallibleConstrain => Associativity::Left,

            // Right-associative operators
            /* Operator::Assign | */ Operator::Implies | Operator::Iff => Associativity::Right, // Check standard practice for Iff

            // Prefix and Postfix are effectively non-associative in the traditional sense
            Operator::Not | Operator::CardinalityOpen | Operator::CardinalityClose => {
                Associativity::None
            }

            // Ternary is special
            Operator::TernaryQ | Operator::TernaryC => Associativity::Right, // Usually right-associative

            // Type constraints might be non-associative
            Operator::TypeConstraint | Operator::HasType => Associativity::None,
        }
    }
}

// --- Pratt Parser Implementation ---

struct PrattParser<'i> {
    pairs: Peekable<Pairs<'i, Rule>>,
}

impl<'i> PrattParser<'i> {
    fn new(pairs: Pairs<'i, Rule>) -> Self {
        PrattParser {
            pairs: pairs.peekable(),
        }
    }

    // Main entry point for parsing an expression
    fn parse_expression(&mut self, min_precedence: Precedence) -> Result<Expression, BorfError> {
        let mut left = self.parse_prefix()?;

        // Loop as long as we find operators with sufficient precedence
        loop {
            // Check for infix operator first
            if let Some(op_info) = self.peek_infix_op_info() {
                if op_info.precedence < min_precedence {
                    break; // Infix precedence too low, stop processing this level
                }

                // Consume the operator pair
                let op_pair = self.consume()?;
                let op = op_info.op; // Already extracted in peek_infix_op_info

                // Adjust precedence for next recursive call based on associativity
                let next_precedence = match op_info.associativity {
                    Associativity::Left => op_info.precedence.next_level(), // For left assoc, require higher precedence on right
                    Associativity::Right => op_info.precedence, // For right assoc, allow same precedence on right
                    Associativity::None => Precedence::Primary, // Should not happen for infix? Or treat as highest?
                };

                // Handle ternary operator specially
                if op == Operator::TernaryQ {
                    left = self.parse_ternary(left, next_precedence)?;
                } else {
                    // Standard infix parsing
                    let right = self.parse_expression(next_precedence)?;
                    left = Expression::BinaryOp {
                        left: Box::new(left),
                        op: op_pair.as_str().to_string(), // Use the original operator string
                        right: Box::new(right),
                    };
                }
            } else if let Some(postfix_op_info) = self.peek_postfix_op_info() {
                // Check for postfix operators like function call or field access
                // These have higher precedence than most infix ops and are left-associative
                if postfix_op_info.precedence < min_precedence {
                    break; // Postfix precedence too low, stop processing this level
                }

                // Consume the operator token ('(', '.', '[')
                self.consume()?; // Consume the peeked token
                let op = postfix_op_info.op;

                // Handle different postfix/circumfix operators
                match op {
                    Operator::FunctionCall => {
                        left = self.parse_function_call_args(left)?;
                    }
                    Operator::FieldAccess => {
                        left = self.parse_field_access(left)?;
                    }
                    Operator::IndexAccess => {
                        left = self.parse_index_access(left)?;
                    }
                    _ => unreachable!("Invalid postfix operator type"),
                }
            } else {
                break;
            }
        }

        Ok(left)
    }

    // Parses prefix operators and primary expressions
    fn parse_prefix(&mut self) -> Result<Expression, BorfError> {
        let pair = self.consume()?;

        match pair.as_rule() {
            // Literals
            Rule::int => Ok(Expression::IntLiteral(pair.as_str().parse().map_err(
                |_| BorfError::ParserError("Invalid integer literal".to_string()),
            )?)),
            Rule::string_literal => Ok(Expression::StringLiteral(
                pair.as_str().trim_matches('"').to_string(),
            )),
            Rule::symbol_literal => Ok(Expression::SymbolLiteral(pair.as_str()[1..].to_string())), // Skip ':'
            Rule::boolean_literal => Ok(Expression::BoolLiteral(pair.as_str() == "true")),

            // Identifiers
            Rule::ident => Ok(Expression::Identifier(Identifier(
                pair.as_str().to_string(),
            ))),
            Rule::dollar_ident => Ok(Expression::DollarIdentifier(pair.as_str().to_string())),
            Rule::qualified_name => {
                let mut parts = pair.as_str().split('.');
                let base = parts.next().unwrap().to_string();
                let field = parts.collect::<Vec<_>>().join("."); // Rejoin if qualified name has multiple parts
                Ok(Expression::QualifiedName(base, field))
            }
            Rule::law_identifier => Ok(Expression::LawIdentifier(pair.as_str().to_string())),

            // Prefix Operators
            Rule::op_prefix_not => {
                let op_str = pair.as_str().to_string();
                let expr = self.parse_expression(Precedence::Prefix)?;
                Ok(Expression::PrefixOp {
                    op: op_str,
                    expr: Box::new(expr),
                })
            }
            Rule::op_prefix_cardinality_open => {
                // This is tricky. Needs to match closing '|'.
                // For now, let's assume it's handled within a specific expr_term rule
                // like `cardinality_expr`. If called directly, it's an error or needs context.
                // If `expression` rule truly flattens it, we need lookahead for the closing '|'.
                // Let's defer proper handling until we see specific test cases fail.
                let expr = self.parse_expression(Precedence::Lowest)?; // Parse inner expression
                if let Some(close_pair) = self.peek() {
                    if close_pair.as_rule() == Rule::op_postfix_cardinality_close {
                        self.consume()?; // Consume the closing '|'
                        Ok(Expression::Cardinality(Box::new(expr)))
                    } else {
                        Err(BorfError::ParserError(
                            "Mismatched cardinality operator '|'".to_string(),
                        ))
                    }
                } else {
                    Err(BorfError::ParserError(
                        "Expected closing '|' for cardinality".to_string(),
                    ))
                }
            }

            // Grouping
            Rule::expression if pair.as_str().starts_with('(') && pair.as_str().ends_with(')') => {
                // This assumes the grammar rule `expression` directly handles `(` expression `)`.
                // We might need a dedicated `grouping` rule in pest.
                // Let's parse the inner content. Need to refine how pairs are handled.
                let _inner_pairs = pair.into_inner();
                // Skip '(' if it's a token, parse inner expression
                // This part is highly dependent on how pest returns the pairs for `(` expression `)`.
                // For now, assuming the inner content is the next logical sequence for the Pratt parser.
                self.parse_expression(Precedence::Lowest) // Re-enter Pratt parser for the content inside ()
            }

            // Other expression terms (Lambda, If, Let, Sets, etc.)
            // These should ideally be parsed here as primary expressions
            Rule::lambda => self.parse_lambda(pair),
            Rule::if_expr => self.parse_if_expr(pair),
            Rule::tuple_expr => self.parse_tuple_expr(pair),
            Rule::set_expr => self.parse_set_expr(pair),
            Rule::record_expr => self.parse_record_expr(pair),
            Rule::let_rec => Err(BorfError::NotYetImplemented {
                feature: "Let rec parsing".to_string(),
                src: None,
                span: None,
            }),
            Rule::set_comprehension => Err(BorfError::NotYetImplemented {
                feature: "Set comprehension parsing".to_string(),
                src: None,
                span: None,
            }),
            // Function Call - needs lookahead for '('
            // Field Access - needs lookahead for '.'
            // Index Access - needs lookahead for '['

            // Fallback for unhandled prefix rules
            _ => Err(BorfError::NotYetImplemented {
                feature: format!("Prefix parsing for rule: {:?}", pair.as_rule()),
                src: None,
                span: None, // TODO: Add span info
            }),
        }
    }

    // Helper to parse ternary ?: operator
    fn parse_ternary(
        &mut self,
        condition: Expression,
        precedence: Precedence,
    ) -> Result<Expression, BorfError> {
        let then_expr = self.parse_expression(precedence)?; // Parse middle expression

        // Expect ':'
        if let Some(pair) = self.peek() {
            if pair.as_rule() == Rule::op_ternary_c
                || (pair.as_rule() == Rule::op_comparison && pair.as_str() == ":")
            {
                self.consume()?; // Consume ':'
            } else {
                return Err(BorfError::ParserError(format!(
                    "Expected ':' for ternary operator, found {:?}",
                    pair.as_rule()
                )));
            }
        } else {
            return Err(BorfError::ParserError(
                "Expected ':' for ternary operator, found end of input".to_string(),
            ));
        }

        let else_expr = self.parse_expression(precedence)?; // Parse last expression

        Ok(Expression::Conditional {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr: Box::new(else_expr),
        })
    }

    // Parses lambda expressions: \x. body or \x y -> body
    fn parse_lambda(&mut self, pair: Pair<'i, Rule>) -> Result<Expression, BorfError> {
        let mut inner = pair.into_inner();
        let mut params = Vec::new();

        // Consume parameters (identifiers)
        while let Some(p) = inner.peek() {
            if p.as_rule() == Rule::ident {
                params.push(inner.next().unwrap().as_str().to_string());
            } else {
                break; // Stop at separator ('.' or '->')
            }
        }

        // Consume separator ('.' or '->') - grammar ensures one exists
        let _separator = inner.next().unwrap();

        // The rest is the body expression
        // We need to collect the remaining pairs representing the body
        // and parse them as a single expression.
        let body_pair = inner
            .next()
            .ok_or_else(|| BorfError::ParserError("Lambda body missing".to_string()))?;

        // Ensure the body_pair is actually an expression rule
        if body_pair.as_rule() == Rule::expression {
            // Create a new PrattParser instance for the body expression's inner tokens
            let body_expr = parse_expression(body_pair)?;
            Ok(Expression::Lambda {
                params,
                body: Box::new(body_expr),
            })
        } else {
            Err(BorfError::ParserError(format!(
                "Expected expression rule for lambda body, found {:?}",
                body_pair.as_rule()
            )))
        }
    }

    // Parses if expressions: if cond then true_branch else false_branch
    fn parse_if_expr(&mut self, pair: Pair<'i, Rule>) -> Result<Expression, BorfError> {
        let mut inner = pair.into_inner(); // Should contain 3 expression pairs
                                           // Order is fixed by grammar: if ~ expression ~ then ~ expression ~ else ~ expression

        let cond_pair = inner
            .next()
            .ok_or_else(|| BorfError::ParserError("If condition missing".to_string()))?;
        let then_pair = inner
            .next()
            .ok_or_else(|| BorfError::ParserError("If 'then' branch missing".to_string()))?;
        let else_pair = inner
            .next()
            .ok_or_else(|| BorfError::ParserError("If 'else' branch missing".to_string()))?;

        // Recursively parse the sub-expressions
        // Need to ensure these pairs are indeed expression rules
        if cond_pair.as_rule() != Rule::expression
            || then_pair.as_rule() != Rule::expression
            || else_pair.as_rule() != Rule::expression
        {
            return Err(BorfError::ParserError(format!(
                "Invalid structure for if expression: {:?}, {:?}, {:?}",
                cond_pair.as_rule(),
                then_pair.as_rule(),
                else_pair.as_rule()
            )));
        }

        let condition = parse_expression(cond_pair)?;
        let then_branch = parse_expression(then_pair)?;
        let else_branch = parse_expression(else_pair)?;

        Ok(Expression::IfThenElse {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    // Parses set literals: { elem1, elem2, ... }
    fn parse_set_expr(&mut self, pair: Pair<'i, Rule>) -> Result<Expression, BorfError> {
        // Grammar: set_expr = { "{" ~ (set_expr_element ~ ("," ~ set_expr_element)*)? ~ "}" }
        // Grammar: set_expr_element = { expression }
        let _inner_pairs = pair.into_inner(); // Should be zero or more set_expr_element pairs
        let elements: Result<Vec<Expression>, BorfError> = _inner_pairs
            .filter(|p| p.as_rule() == Rule::set_expr_element)
            .map(|elem_pair| {
                // Each element is an expression
                let expr_pair = elem_pair
                    .into_inner()
                    .next()
                    .ok_or_else(|| BorfError::ParserError("Empty set_expr_element".to_string()))?;
                if expr_pair.as_rule() == Rule::expression {
                    parse_expression(expr_pair)
                } else {
                    Err(BorfError::ParserError(format!(
                        "Expected expression in set literal element, found {:?}",
                        expr_pair.as_rule()
                    )))
                }
            })
            .collect();

        elements.map(Expression::SetLiteral) // Wrap the Vec<Expression> in the AST node
    }

    // Parses tuple literals: (elem1, elem2, ...)
    fn parse_tuple_expr(&mut self, pair: Pair<'i, Rule>) -> Result<Expression, BorfError> {
        // Grammar: tuple_expr = { "(" ~ expression ~ ("," ~ expression)+ ~ ")" }
        let _inner_pairs = pair.into_inner(); // Should be expression pairs
        let elements: Result<Vec<Expression>, BorfError> = _inner_pairs
            .filter(|p| p.as_rule() == Rule::expression)
            .map(parse_expression) // Each inner pair should be an expression
            .collect();

        elements.map(Expression::Tuple)
    }

    // Parses record literals: { field1 = val1, field2 = val2, ... }
    fn parse_record_expr(&mut self, pair: Pair<'i, Rule>) -> Result<Expression, BorfError> {
        // Grammar: record_expr = { "{" ~ (record_expr_entry ~ ("," ~ record_expr_entry)*)? ~ "}" }
        // Grammar: record_expr_entry = { ident ~ "=" ~ expression }
        let _inner_pairs = pair.into_inner(); // Should be zero or more record_expr_entry pairs
        let entries: Result<Vec<(String, Expression)>, BorfError> = _inner_pairs
            .filter(|p| p.as_rule() == Rule::record_expr_entry)
            .map(|entry_pair| {
                let mut entry_inner = entry_pair.into_inner(); // ident, expression
                let ident_pair = entry_inner.next().ok_or_else(|| {
                    BorfError::ParserError("Missing identifier in record entry".to_string())
                })?;
                let expr_pair = entry_inner.next().ok_or_else(|| {
                    BorfError::ParserError("Missing expression in record entry".to_string())
                })?;

                if ident_pair.as_rule() == Rule::ident && expr_pair.as_rule() == Rule::expression {
                    let value_expr = parse_expression(expr_pair)?;
                    Ok((ident_pair.as_str().to_string(), value_expr))
                } else {
                    Err(BorfError::ParserError(format!(
                        "Invalid record entry structure: {:?}, {:?}",
                        ident_pair.as_rule(),
                        expr_pair.as_rule()
                    )))
                }
            })
            .collect();

        // TODO: Add Expression::RecordLiteral(Vec<(String, Expression)>) to ast.rs
        // Map to placeholder LetRec for now
        entries.map(|bindings| Expression::LetRec {
            bindings,
            body: Box::new(Expression::Identifier(Identifier(
                "RecordPlaceholderBody".to_string(),
            ))),
        })
    }

    // Peek at the next token to see if it's an infix operator
    fn peek_infix_op_info(&mut self) -> Option<OpInfo> {
        self.pairs.peek().and_then(|pair| {
            Operator::from_pair(pair).map(|op| OpInfo {
                op,
                precedence: op.precedence(),
                associativity: op.associativity(),
            })
        })
    }

    // Peek at the next pair without consuming
    fn peek(&mut self) -> Option<&Pair<'i, Rule>> {
        self.pairs.peek()
    }

    // Consume the next pair
    fn consume(&mut self) -> Result<Pair<'i, Rule>, BorfError> {
        self.pairs
            .next()
            .ok_or_else(|| BorfError::ParserError("Unexpected end of input".to_string()))
    }

    // Parses function call arguments after the function expression has been parsed
    // Expects the opening '(' to have been consumed already.
    fn parse_function_call_args(&mut self, func_expr: Expression) -> Result<Expression, BorfError> {
        let mut args = Vec::new();
        // Check if next token is closing ')' for zero-arg call
        if let Some(peeked) = self.peek() {
            if peeked.as_str() == ")" {
                self.consume()?; // Consume ')'
                return Ok(Expression::FunctionCall {
                    func: Box::new(func_expr),
                    args,
                });
            }
        }

        // Parse comma-separated arguments until ')'
        loop {
            // Parse an argument expression
            let arg_expr = self.parse_expression(Precedence::Lowest)?;
            args.push(arg_expr);

            // Check for ',' or ')'
            if let Some(peeked) = self.peek() {
                match peeked.as_str() {
                    ")" => {
                        self.consume()?; // Consume ')'
                        break; // End of arguments
                    }
                    "," => {
                        self.consume()?; // Consume ',' and continue parsing next arg
                    }
                    _ => {
                        return Err(BorfError::ParserError(format!(
                            "Expected ',' or ')' in function arguments, found: {}",
                            peeked.as_str()
                        )));
                    }
                }
            } else {
                return Err(BorfError::ParserError(
                    "Unexpected end of input in function arguments, expected ')'".to_string(),
                ));
            }
        }

        Ok(Expression::FunctionCall {
            func: Box::new(func_expr),
            args,
        })
    }

    // Parses field access after the base expression has been parsed
    // Expects the '.' to have been consumed already.
    fn parse_field_access(&mut self, base_expr: Expression) -> Result<Expression, BorfError> {
        if let Some(field_pair) = self.peek() {
            if field_pair.as_rule() == Rule::ident {
                let field_name = field_pair.as_str().to_string();
                self.consume()?; // Consume identifier

                Ok(Expression::FieldAccess {
                    base: Box::new(base_expr),
                    field: field_name,
                })
            } else {
                Err(BorfError::ParserError(format!(
                    "Expected field name (ident) after '.' but found {:?}",
                    field_pair.as_rule()
                )))
            }
        } else {
            Err(BorfError::ParserError(
                "Expected field name after '.' but found end of input".to_string(),
            ))
        }
    }

    // Peek at the next token and determine if it signals a postfix operator (call, field access, index)
    fn peek_postfix_op_info(&mut self) -> Option<OpInfo> {
        self.pairs
            .peek()
            .and_then(|pair| {
                match pair.as_str() {
                    // Match on the literal token text
                    "(" => Some(Operator::FunctionCall),
                    "." => Some(Operator::FieldAccess),
                    "[" => Some(Operator::IndexAccess),
                    _ => None,
                }
            })
            .map(|op| OpInfo {
                // Map the Operator to OpInfo
                op,
                precedence: op.precedence(),
                associativity: op.associativity(),
            })
    }

    // Parses index access: base[index]
    // Expects '[' to have been consumed.
    fn parse_index_access(&mut self, base_expr: Expression) -> Result<Expression, BorfError> {
        let index_expr = self.parse_expression(Precedence::Lowest)?;
        // Expect closing ']'
        if let Some(pair) = self.peek() {
            if pair.as_str() == "]" {
                self.consume()?; // Consume ']'
                Ok(Expression::IndexAccess {
                    base: Box::new(base_expr),
                    index: Box::new(index_expr),
                })
            } else {
                Err(BorfError::ParserError(format!(
                    "Expected ']' for index access, found {:?}",
                    pair.as_rule()
                )))
            }
        } else {
            Err(BorfError::ParserError(
                "Unexpected end of input after index expression, expected ']'".to_string(),
            ))
        }
    }
}

// Helper struct to hold operator info
#[derive(Debug, Clone, Copy)]
struct OpInfo {
    op: Operator,
    precedence: Precedence,
    associativity: Associativity,
}

// Helper to get the next precedence level (useful for left associativity)
impl Precedence {
    fn next_level(self) -> Precedence {
        use Precedence::*;
        match self {
            Lowest => Assignment,
            Assignment => Ternary,
            Ternary => Or,
            Or => And,
            And => Iff,
            Iff => Implies,
            Implies => Equality,
            Equality => Comparison,
            Comparison => Set,
            Set => Term,
            Term => Factor,
            Factor => Fallible,
            Fallible => Arrow,
            Arrow => TransitiveClosure,
            TransitiveClosure => Compose,
            Compose => Prefix,
            Prefix => Postfix,
            Postfix => Primary,
            Primary => Primary, // No higher precedence
        }
    }
}

// --- Public API ---

// Replace the old stub function
pub fn parse_expression(pair: Pair<Rule>) -> Result<Expression, BorfError> {
    // Need to handle the outer `expression` or `expr_term` rule if present
    let _inner_pairs = if pair.as_rule() == Rule::expression {
        // If the grammar rule `expression = { expr_term ~ (operator ~ expr_term)* }`
        // then `pair.into_inner()` will give a sequence of `expr_term` and `operator`.
        pair.into_inner()
    } else if pair.as_rule() == Rule::expr_term {
        // If we are passed an expr_term directly, its inner pairs might be more complex
        // e.g. for `atom ~ "(" ...` (function call) or `(` expression `)`.
        // Let's try just passing this single pair's children.
        // This needs testing. A cleaner approach might be to always start parsing
        // from the top-level `expression` rule.
        // For now, let's assume the caller handles providing the correct stream.
        // If pair is expr_term containing just an atom, its inner pairs might be just that atom.
        pair.into_inner()
    } else {
        // If we get something completely different (like 'ident' directly),
        // this approach won't work well. The Pratt parser expects a stream.
        return Err(BorfError::ParserError(format!(
             "parse_expression called with unexpected rule: {:?}. Expected 'expression' or 'expr_term'.",
             pair.as_rule()
         )));
        // An alternative for single tokens, though Pratt isn't ideal for it:
        // return PrattParser::new(vec![pair].into_iter().peekable()).parse_expression(Precedence::Lowest);
    };

    let mut parser = PrattParser::new(_inner_pairs);
    let result = parser.parse_expression(Precedence::Lowest)?;

    // Check if the parser consumed all tokens provided to it.
    // If not, it might indicate an error or incomplete parse.
    if parser.peek().is_some() {
        // This might be okay if the caller only passed a subset of pairs,
        // but often indicates a problem in parsing logic or grammar matching.
        // Let's return an error for now, can be refined.
        let remaining_pair = parser.peek().unwrap();
        return Err(BorfError::ParserError(format!(
            "Pratt parser did not consume all input. Remaining token: {:?} ('{}')",
            remaining_pair.as_rule(),
            remaining_pair.as_str()
        )));
    }

    Ok(result)
}

// Type expression operators for the Pratt parser
#[derive(Debug, Clone, Copy)]
enum TypeExprOp {
    Product,           // *
    Sum,               // +
    Function,          // ->
    LinearFunction,    // -o
    TransitiveClosure, // ->+
    Map,               // :->
    Union,             // $cup
    Intersection,      // $cap
}

// Parse a type operator from a rule
fn parse_type_op(pair: &Pair<Rule>) -> Option<(Precedence, TypeExprOp)> {
    match pair.as_rule() {
        Rule::type_op_product => Some((Precedence::Factor, TypeExprOp::Product)),
        Rule::type_op_sum => Some((Precedence::Term, TypeExprOp::Sum)),
        Rule::type_op_arrow => match pair.as_str() {
            "->" => Some((Precedence::Arrow, TypeExprOp::Function)),
            "->+" => Some((Precedence::TransitiveClosure, TypeExprOp::TransitiveClosure)),
            "-o" => Some((Precedence::Arrow, TypeExprOp::LinearFunction)),
            _ => None,
        },
        Rule::type_op_map => Some((Precedence::Arrow, TypeExprOp::Map)),
        Rule::type_op_cup => Some((Precedence::Set, TypeExprOp::Union)),
        Rule::type_op_cap => Some((Precedence::Set, TypeExprOp::Intersection)),
        _ => None,
    }
}

// Apply a type operator to two type expressions
fn apply_type_op(op: TypeExprOp, left: TypeExpr, right: TypeExpr) -> TypeExpr {
    match op {
        TypeExprOp::Product => TypeExpr::Product(Box::new(left), Box::new(right)),
        TypeExprOp::Sum => TypeExpr::Sum(Box::new(left), Box::new(right)),
        TypeExprOp::Function => TypeExpr::Function(Box::new(left), Box::new(right)),
        TypeExprOp::LinearFunction => TypeExpr::LinearFunction(Box::new(left), Box::new(right)),
        TypeExprOp::TransitiveClosure => TypeExpr::MultiParamFunction(vec![left], Box::new(right)),
        TypeExprOp::Map => TypeExpr::Map(Box::new(left), Box::new(right)),
        TypeExprOp::Union => TypeExpr::Union(Box::new(left), Box::new(right)),
        TypeExprOp::Intersection => TypeExpr::Intersection(Box::new(left), Box::new(right)),
    }
}

// Keep the TypeExpr stub for now, or implement a similar Pratt parser for types
pub fn parse_type_expression(pair: Pair<Rule>) -> Result<TypeExpr, BorfError> {
    // TODO: Implement Pratt parser for TypeExpr if needed, handling ->, *, +, etc.
    match pair.as_rule() {
        Rule::type_expr => {
            // For now, just parse the first term found. This won't handle infix ops.
            // A full type parser is needed here.
            let first_term = pair.into_inner().find(|p| {
                p.as_rule() != Rule::type_infix_op
                    && p.as_rule() != Rule::COMMENT
                    && p.as_rule() != Rule::WHITESPACE
            });
            if let Some(term) = first_term {
                parse_type_expression(term)
            } else {
                Ok(TypeExpr::TypeName("UnknownTypeExpr".to_string())) // Placeholder
            }
            // Old logic:
            // let inner = pair.into_inner().next().unwrap();
            // parse_type_expression(inner) // Recurse for now
        }
        Rule::ident | Rule::dollar_ident | Rule::Sym | Rule::X | Rule::Any | Rule::Pattern => {
            Ok(TypeExpr::TypeName(pair.as_str().to_string()))
        }
        Rule::qualified_name => {
            let mut parts = pair.as_str().splitn(2, '.');
            let mod_name = parts.next().unwrap_or("").to_string();
            let type_name = parts.next().unwrap_or("").to_string();
            Ok(TypeExpr::QualifiedName(mod_name, type_name))
        }
        Rule::sequence_type => {
            // Handle Seq T syntax
            let inner_type_pair = pair
                .into_inner()
                .find(|p| p.as_rule() == Rule::type_term)
                .ok_or_else(|| {
                    BorfError::ParserError("Expected type_term after Seq".to_string())
                })?;
            let inner_type = parse_type_expression(inner_type_pair)?;
            Ok(TypeExpr::Sequence(Box::new(inner_type)))
        }
        Rule::set_type => {
            // Assuming set_type = { "{" ~ type_expr ~ "}" }
            let inner_type_pair = pair
                .into_inner()
                .find(|p| p.as_rule() == Rule::type_expr)
                .ok_or_else(|| {
                    BorfError::ParserError("Expected type_expr inside set_type {}".to_string())
                })?;
            let inner_type = parse_type_expression(inner_type_pair)?;
            Ok(TypeExpr::Set(Box::new(inner_type)))
        }
        Rule::list_type => {
            // Assuming list_type = { "[" ~ type_expr ~ "]" }
            let inner_type_pair = pair
                .into_inner()
                .find(|p| p.as_rule() == Rule::type_expr)
                .ok_or_else(|| {
                    BorfError::ParserError("Expected type_expr inside list_type []".to_string())
                })?;
            let inner_type = parse_type_expression(inner_type_pair)?;
            Ok(TypeExpr::List(Box::new(inner_type)))
        }
        Rule::tuple_type => {
            // Assuming tuple_type = { "(" ~ type_expr ~ ("," ~ type_expr)+ ~ ")" }
            let types: Result<Vec<_>, _> = pair
                .into_inner()
                .filter(|p| p.as_rule() == Rule::type_expr)
                .map(parse_type_expression)
                .collect();
            Ok(TypeExpr::Tuple(types?))
        }
        Rule::record_type => {
            // Assuming record_type = { "{" ~ (record_type_entry ~ ("," ~ record_type_entry)*)? ~ "}" }
            let entries: Result<Vec<(String, TypeExpr)>, BorfError> = pair
                .into_inner()
                .filter(|p| p.as_rule() == Rule::record_type_entry)
                .map(|entry_pair| {
                    let mut inner = entry_pair.into_inner();
                    let name_pair = inner.next().ok_or_else(|| {
                        BorfError::ParserError("Expected name in record type entry".to_string())
                    })?;
                    // Skip ':' if it's explicit token
                    // let _colon = inner.next();
                    let type_pair = inner.next().ok_or_else(|| {
                        BorfError::ParserError("Expected type in record type entry".to_string())
                    })?;
                    let type_expr = parse_type_expression(type_pair)?;
                    Ok((name_pair.as_str().to_string(), type_expr))
                })
                .collect();
            Ok(TypeExpr::Record(entries?))
        }
        Rule::optional_type => {
            // Assuming optional_type = { "?" ~ type_term }
            let inner_type_pair = pair
                .into_inner()
                .find(|p| p.as_rule() == Rule::type_term)
                .ok_or_else(|| {
                    BorfError::ParserError("Expected type_term after '?'".to_string())
                })?;
            let inner_type = parse_type_expression(inner_type_pair)?;
            Ok(TypeExpr::Optional(Box::new(inner_type)))
        }
        Rule::linear_type => {
            // Assuming linear_type = { "!" ~ type_term }
            let inner_type_pair = pair
                .into_inner()
                .find(|p| p.as_rule() == Rule::type_term)
                .ok_or_else(|| {
                    BorfError::ParserError("Expected type_term after '!'".to_string())
                })?;
            let inner_type = parse_type_expression(inner_type_pair)?;
            Ok(TypeExpr::Linear(Box::new(inner_type)))
        }
        // Add more TypeExpr cases as needed based on AST and grammar
        _ => {
            let rule_str = format!("{:?}", pair.as_rule());
            // Check if it's a type term that needs further recursion
            if [Rule::type_term, Rule::constraint_expr /* etc */].contains(&pair.as_rule()) {
                // If it's a wrapper rule, try parsing its contents
                if let Some(inner_pair) = pair.into_inner().next() {
                    return parse_type_expression(inner_pair);
                }
            }
            // Fallback placeholder
            Ok(TypeExpr::TypeName(format!("UnparsedType_{}", rule_str)))
        }
    }
}
