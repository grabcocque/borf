use crate::eval_str;

// Parsing is integrated with evaluation; test via eval_str.

#[test]
fn test_eval_add_int() {
    let out = eval_str("1+2").unwrap();
    assert_eq!(out, "3");
}

#[test]
fn test_eval_mul_precedence() {
    let out = eval_str("1+2*3").unwrap();
    assert_eq!(out, "7");
}

#[test]
fn test_eval_parentheses() {
    let out = eval_str("(1+2)*3").unwrap();
    assert_eq!(out, "9");
}

#[test]
fn test_eval_div_float() {
    let out = eval_str("7/2").unwrap();
    assert_eq!(out, "3.5");
}
