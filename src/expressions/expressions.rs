use std::collections::HashMap;
use std::{fmt, vec};
use super::vec_types::Vector;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Token {
    Binding(usize),
    IntValue(i64),
    FloatValue(f64),
    BracketOpen,
    BracketClose,
    Comma,
    CurlyOpen,
    CurlyClose,
    BinaryPlus,
    UnaryPlus,
    BinaryMinus,
    UnaryMinus,
    Multiply,
    Divide,
    VectorValue(Vector<f64, 3>),
    MakeVec(usize),
    VecIndex(usize),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Binding(x) => write!(f, "[bind@{}]", x),
            Token::IntValue(x) => write!(f, "[int value:{}]", x),
            Token::FloatValue(x) => write!(f, "[int value:{}]", x),
            Token::VectorValue(x) => write!(f, "[vec value:{{{},{},{}}}]", x[0], x[1], x[2]),
            Token::BracketOpen => write!(f, "[(]"),
            Token::BracketClose => write!(f, "[)]"),
            Token::Comma => write!(f, "[,]"),
            Token::CurlyOpen => write!(f, "[{{]"),
            Token::CurlyClose => write!(f, "[}}]"),
            Token::BinaryPlus => write!(f, "[binary+]"),
            Token::UnaryPlus => write!(f, "[unary+]"),
            Token::BinaryMinus => write!(f, "[binary-]"),
            Token::UnaryMinus => write!(f, "[unary-]"),
            Token::Multiply => write!(f, "[*]"),
            Token::Divide => write!(f, "[/]"),
            Token::MakeVec(size) => write!(f, "[mkvec{}]", size),
            Token::VecIndex(i) => write!(f, "[vindex.{}]", i),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BindingValue {
    Unknown,
    Int(i64),
    Float(f64),
    Vector3(Vector<f64, 3>),
}

#[derive(Clone)]
pub struct Binding {
    pub name: String,
    pub value: BindingValue,
}

pub struct PrecompiledCode {
    postfix: Vec<Token>,
    bindings: Vec<Binding>
}

impl PrecompiledCode {
    pub fn binding_names<'a>(&'a self) -> Vec<&'a str> {
        self.bindings.iter().map(|b| { b.name.as_str() }).collect()
    }

    pub fn binding_map_to_values(&self, binding_map: &HashMap<String, BindingValue>) -> Vec<BindingValue> {
        self.bindings.iter().filter_map(|b| {
            if let Some(value) = binding_map.get(&b.name) {
                Some(*value)
            } else {
                None
            }
        }).collect()
    }

    pub fn clone_binding_values(&self) -> Vec<BindingValue> {
        self.bindings.iter().map(|x| { x.value }).collect()
    }

    pub fn set_bindings_from_map(&mut self, binding_map: &HashMap<String, BindingValue>) {
        for binding in self.bindings.iter_mut() {
            if let Some(value) = binding_map.get(&binding.name) {
                binding.value = *value;
            }
        }
    }

    pub fn set_bindings_from_vec(&mut self, bindings: &Vec<BindingValue>) {
        for (binding, value) in self.bindings.iter_mut().zip(bindings.iter()) {
            binding.value = *value;
        }
    }
}


/// errors
/// 

#[derive(Debug)]
pub enum ExpressionError {
    CompilationError(CompilationError),
    EvaluationError(EvaluationError),
}

pub struct CompilationError {
    message: String,
}

impl fmt::Display for CompilationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CompilationError: {}", self.message)
    }
}

impl fmt::Debug for CompilationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<COMPILATION ERROR HAPPENED: {}>", self.message)
    }
}

pub struct EvaluationError {
    message: String,
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EvaluationError: {}", self.message)
    }
}

impl fmt::Debug for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<EVALUATION ERROR HAPPENED: {}>", self.message)
    }
}

///

#[derive(Debug, Copy, Clone)]
enum ValueType {
    NotInitialized,
    Int,
    Float,
    Binding,
    DotIndex,
}

/// parses human readable expression into a token array
/// returns token vector, and a map of binding nums to binding names
fn tokenize(line: &str) -> Result<(Vec<Token>, Vec<Binding>), CompilationError> {
    let mut bindings = Vec::new();

    let mut line_tokens: Vec<Token> = Vec::new();
    let mut token_start = 0;
    let mut token_end = 0;
    let mut isval = ValueType::NotInitialized;

    macro_rules! finalize_token_if_started {
        () => {
            match isval {
                ValueType::Int => {
                    line_tokens.push(Token::IntValue(
                        line[token_start..=token_end]
                            .parse()
                            .expect(format!("bad int '{:?}'", &line[token_start..=token_end]).as_str()),
                    ));
                }
                ValueType::Float => {
                    line_tokens.push(Token::FloatValue(
                        line[token_start..=token_end].parse().expect("bad float"),
                    ));
                }
                ValueType::Binding => {
                    let token_name = line[token_start..=token_end].to_owned();
                    bindings.push(
                        Binding {
                            name: token_name,
                            value: BindingValue::Unknown,
                        },
                    );
                    line_tokens.push(Token::Binding(bindings.len() - 1));
                }
                ValueType::DotIndex => {
                    match &line[token_start..=token_end] {
                        "x" => line_tokens.push(Token::VecIndex(0)),
                        "y" => line_tokens.push(Token::VecIndex(1)),
                        "z" => line_tokens.push(Token::VecIndex(2)),
                        "w" => line_tokens.push(Token::VecIndex(3)),
                        c => {
                            return Err(CompilationError {
                                message: format!("dot (.) index can only be one of x y z w, but '{}' found", c)
                            })
                        }
                    }
                }
                ValueType::NotInitialized => {
                    // nothing, no token started
                }
            }
            isval = ValueType::NotInitialized;
        };
    }

    for (i, c) in line.trim().chars().enumerate() {
        match (isval, c, line_tokens.last()) {
            (_, c, _) if c.is_ascii_whitespace() => {
                finalize_token_if_started!();
            }
            (ValueType::NotInitialized, '.', Some(Token::Binding(_)) | Some(Token::CurlyClose) | Some(Token::BracketClose)) => {
                token_start = i + 1;
                isval = ValueType::DotIndex;
            }
            (ValueType::NotInitialized | ValueType::Float | ValueType::Int, '0'..='9' | '.', _) => {
                match isval {
                    ValueType::NotInitialized => {
                        isval = if c == '.' { ValueType::Float } else { ValueType::Int };
                        token_start = i;
                    }
                    ValueType::Int if c == '.' => {
                        isval = ValueType::Float;
                    }
                    _ => (),
                }
                token_end = i;
            }
            (ValueType::NotInitialized, '@', _) => {
                isval = ValueType::Binding;
                token_start = i + 1;
            }
            (ValueType::Binding, c, _) if c.is_ascii_alphanumeric() => {
                token_end = i;
            }
            (ValueType::DotIndex, c, _) if c.is_alphabetic() => {
                token_end = i;
            }
            (_, c, _) => {
                // first finalize any numeric token if any started
                if let ValueType::NotInitialized = isval {
                } else {
                    finalize_token_if_started!();
                }

                line_tokens.push(match c {
                    '+' => match line_tokens.last() {
                        Some(Token::IntValue(_) | Token::FloatValue(_) | Token::Binding(_) | Token::BracketClose | Token::CurlyClose | Token::VecIndex(_)) => {
                            Token::BinaryPlus
                        }
                        _ => Token::UnaryPlus,
                    },
                    '-' => match line_tokens.last() {
                        Some(Token::IntValue(_) | Token::FloatValue(_) | Token::Binding(_) | Token::BracketClose | Token::CurlyClose | Token::VecIndex(_)) => {
                            Token::BinaryMinus
                        }
                        _ => Token::UnaryMinus,
                    },
                    '*' => Token::Multiply,
                    '/' => Token::Divide,
                    '(' => Token::BracketOpen,
                    ')' => Token::BracketClose,
                    '{' => Token::CurlyOpen,
                    '}' => Token::CurlyClose,
                    ',' => Token::Comma,
                    '.' => {
                        token_start = i + 1;
                        isval = ValueType::DotIndex;
                        continue;
                    }
                    _ => {
                        return Err(CompilationError {
                            message: format!("bad token symbol '{}'", c),
                        })
                    }
                });
            }
        }
    }
    finalize_token_if_started!();

    Ok((line_tokens, bindings))
}

fn to_postfix(tokens_sequence: Vec<Token>) -> Result<Vec<Token>, CompilationError> {
    let mut stack: Vec<Token> = Vec::new();
    let mut result: Vec<Token> = Vec::new();
    for token in tokens_sequence {
        // println!("{:?} [[{:?}", token, stack);
        match token {
            Token::IntValue(_) | Token::FloatValue(_) | Token::VectorValue(_) | Token::Binding(_) | Token::VecIndex(_) => result.push(token),
            Token::UnaryPlus | Token::UnaryMinus => stack.push(token),
            Token::BracketOpen => stack.push(Token::BracketOpen),
            Token::BracketClose => loop {
                match stack.pop() {
                    Some(Token::BracketOpen) => break,
                    Some(Token::CurlyOpen) => {
                        return Err(CompilationError {
                            message: format!("bracket mismatch, {{"),
                        })
                    }
                    Some(x) => result.push(x),
                    None => {
                        return Err(CompilationError {
                            message: format!("bracket mismatch!"),
                        })
                    }
                };
            },
            Token::Comma => {
                loop {
                    match stack.last() {
                        Some(Token::UnaryPlus)
                        | Some(Token::UnaryMinus)
                        | Some(Token::Multiply)
                        | Some(Token::Divide)
                        | Some(Token::BinaryPlus)
                        | Some(Token::BinaryMinus) => {
                            result.push(stack.pop().expect("this is imposibru"));
                        }
                        _ => break,
                    };
                }
                stack.push(token)  // so that we count them after
            }
            Token::CurlyOpen => stack.push(token),
            Token::CurlyClose => {
                let mut elem_count = 1;
                loop {
                    match stack.pop() {
                        Some(Token::CurlyOpen) => {
                            result.push(Token::MakeVec(elem_count));
                            break;
                        }
                        Some(Token::BracketOpen) => {
                            return Err(CompilationError {
                                message: format!("bracket mismatch, ("),
                            })
                        }
                        Some(Token::Comma) => {
                            elem_count += 1;
                        }
                        Some(x) => result.push(x),
                        None => {
                            return Err(CompilationError {
                                message: format!("curly bracket mismatch"),
                            })
                        }
                    }
                }
            }
            Token::BinaryPlus | Token::BinaryMinus => {
                loop {
                    match stack.last() {
                        Some(Token::UnaryPlus)
                        | Some(Token::UnaryMinus)
                        | Some(Token::Multiply)
                        | Some(Token::Divide)
                        | Some(Token::BinaryPlus)
                        | Some(Token::BinaryMinus) => {
                            result.push(stack.pop().expect("this is imposibru"));
                        }
                        _ => break,
                    };
                }
                stack.push(token);
            }
            Token::Multiply | Token::Divide => {
                loop {
                    match stack.last() {
                        Some(Token::UnaryPlus) | Some(Token::UnaryMinus) | Some(Token::Multiply) | Some(Token::Divide) => {
                            result.push(stack.pop().expect("this is imposibru"));
                        }
                        _ => break,
                    };
                }
                stack.push(token);
            }
            _ => {
                panic!("TODO: tokens should be split, some are not supposed to happen at this stage")
            }
        }
    }
    while stack.len() > 0 {
        match stack.pop() {
            Some(x) => result.push(x),
            None => panic!("this is impossibru2!"),
        }
    }
    Ok(result)
}

///
/// check syntax correctness
fn validate_postfix(postfix_sequence: &Vec<Token>) -> Result<(), CompilationError> {
    let mut stack_size: i64 = 0;
    for token in postfix_sequence.iter() {
        match token {
            Token::IntValue(_) | Token::FloatValue(_) | Token::VectorValue(_) | Token::Binding(_) => stack_size += 1,
            Token::UnaryMinus | Token::UnaryPlus | Token::VecIndex(_) => (),  // minus 1, put back 1
            Token::BinaryMinus | Token::BinaryPlus | Token::Multiply | Token::Divide => stack_size -= 1,  // minus 2, put back 1
            Token::MakeVec(x) => stack_size += -(*x as i64)+1,
            x => {
                return Err(CompilationError {
                    message: format!("Unexpected token in postfix sequence: '{}'", x)
                });
            }
        }
        if stack_size <= 0 {
            return Err(CompilationError {
                message: format!("not enough operands for operation: '{}'", token)
            });
        }
    }
    if stack_size != 1 {
        return Err(CompilationError {
            message: format!("too many values in stack! {}", stack_size)
        });
    }
    Ok(())
}

///
fn evaluate_postfix(postfix_sequence: &Vec<Token>, bindings: &Vec<BindingValue>) -> Result<BindingValue, EvaluationError> {
    let mut stack: Vec<Token> = Vec::new();
    for token in postfix_sequence.iter() {
        match token {
            Token::IntValue(_) | Token::FloatValue(_) => stack.push(*token),
            Token::Binding(b) => {
                let binding = bindings[*b as usize];
                match binding {
                    BindingValue::Float(x) => stack.push(Token::FloatValue(x)),
                    BindingValue::Int(x) => stack.push(Token::IntValue(x)),
                    BindingValue::Vector3(v) => stack.push(Token::VectorValue(v)),
                    BindingValue::Unknown => {
                        return Err(EvaluationError {
                            message: format!("binding {:?} is not set", b),
                            // TODO: error should provide binding num so that wrapping func may
                            //  present a better formed error message
                        });
                    }
                }
            }
            Token::VecIndex(i) => {
                match stack.pop() {
                    Some(Token::VectorValue(v)) => stack.push(Token::FloatValue(v[*i])),
                    Some(_) => {
                        return Err(EvaluationError {
                            message: format!("Only vector types support .x .y .z .w indexing"),
                        })
                    }
                    _ => {
                        return Err(EvaluationError {
                            message: format!("bad postfix: misplaced unary plus"),
                        })
                    }
                };
            }
            Token::UnaryPlus => {
                match stack.pop() {
                    Some(Token::IntValue(x)) => stack.push(Token::IntValue(x)),
                    Some(Token::FloatValue(x)) => stack.push(Token::FloatValue(x)),
                    Some(Token::VectorValue(x)) => stack.push(Token::VectorValue(x)),
                    _ => {
                        return Err(EvaluationError {
                            message: format!("bad postfix: misplaced unary plus"),
                        })
                    }
                };
            }
            Token::UnaryMinus => {
                match stack.pop() {
                    Some(Token::IntValue(x)) => stack.push(Token::IntValue(-x)),
                    Some(Token::FloatValue(x)) => stack.push(Token::FloatValue(-x)),
                    Some(Token::VectorValue(x)) => stack.push(Token::VectorValue(x*-1.)),
                    x => {
                        return Err(EvaluationError {
                            message: format!("bad postfix: misplaced unary minus in front of {:?}", x),
                        })
                    }
                };
            }
            Token::BinaryPlus | Token::BinaryMinus | Token::Multiply | Token::Divide => {
                let val2 = stack.pop();
                let val1 = stack.pop();

                let mut matched = false;

                macro_rules! sum_helper {
                    (
                        $( $opt1:tt, $opt2:tt, $res:tt, $rt1:ty, $rt2:ty )|* ,
                        $( $iopt1:tt, $iopt2:tt, $ires:tt, $irt1:ty, $irt2:ty )|*
                    ) => {
                        match (val1, val2) {
                        $(
                            (Some(Token::$opt1(x)), Some(Token::$opt2(y))) => {
                                matched = true;
                                stack.push(Token::$res(match token {
                                    Token::BinaryPlus => x as $rt1 + y as $rt2,
                                    Token::BinaryMinus => x as $rt1 - y as $rt2,
                                    Token::Multiply => x as $rt1 * y as $rt2,
                                    Token::Divide => x as $rt1 / y as $rt2,
                                    _ => panic!("impossibruuuu!!"),
                                }));
                            }
                        )*
                        $(
                            (Some(Token::$iopt1(x)), Some(Token::$iopt2(y))) => {
                                matched = true;
                                stack.push(Token::$ires(match token {
                                    Token::BinaryPlus => y as $irt2 + x as $irt1,
                                    Token::BinaryMinus => y as $irt2 - x as $irt1,
                                    Token::Multiply => y as $irt2 * x as $irt1,
                                    Token::Divide => y as $irt2 / x as $irt1,
                                    _ => panic!("impossibruuuu!!"),
                                }));
                            }
                        )*
                            (Some(x), Some(y)) => {
                                return Err(EvaluationError {
                                    message: format!(
                                        "bad expression: operation {} is not defined for arguments {} and {}",
                                        token,
                                        x,
                                        y
                                    )
                                })
                            },
                            _ => {
                                return Err(EvaluationError {
                                    message: format!("bad postfix: not enough operands for {}", token),
                                })
                            }
                        };
                    };
                }
                sum_helper!(
                    IntValue, IntValue, IntValue, i64, i64 |
                    IntValue, FloatValue, FloatValue, f64, f64 |
                    FloatValue, IntValue, FloatValue, f64, f64 |
                    FloatValue, FloatValue, FloatValue, f64, f64 |
                    VectorValue, FloatValue, VectorValue, Vector<f64, 3>, f64 |
                    VectorValue, IntValue, VectorValue, Vector<f64, 3>, f64 |
                    VectorValue, VectorValue, VectorValue, Vector<f64, 3>, Vector<f64, 3>,
                    // now swapped arg ops
                    IntValue, VectorValue, VectorValue, f64, Vector<f64, 3> |
                    FloatValue, VectorValue, VectorValue, f64, Vector<f64, 3>
                );

                if !matched {
                    return Err(EvaluationError {
                        message: format!("cannot perform {:?} for types {:?} and {:?}", token, val1, val2)
                    });
                }
            }
            Token::MakeVec(vec_size) => {
                macro_rules! poporelse {
                    ($stack:ident) => {
                        match $stack.pop() {
                            Some(Token::FloatValue(x)) => x,
                            Some(Token::IntValue(x)) => x as f64,
                            Some(x) => {
                                return Err(EvaluationError {
                                    message: format!("bad argument for operation: {:?}", x)
                                }); 
                            }
                            None => {
                                return Err(EvaluationError {
                                    message: format!("insufficient arguments for operation")
                                }); 
                            }
                        }
                    };
                }
                match vec_size {
                    3 => {
                        let arg3 = poporelse!(stack);
                        let arg2 = poporelse!(stack);
                        let arg1 = poporelse!(stack);
                        stack.push(Token::VectorValue(Vector::new(arg1, arg2, arg3)));
                    }
                    _ => { return Err(EvaluationError {
                        message: format!("vectors of size {} are not supported", vec_size)
                    }); 
                }
                }
            }
            _ => panic!("there should not be brackets in polish postfix notaion. maybe use a special token type?"),
        }
    }
    match stack.pop() {
        Some(Token::IntValue(x)) => Ok(BindingValue::Int(x)),
        Some(Token::FloatValue(x)) => Ok(BindingValue::Float(x)),
        Some(Token::VectorValue(x)) => Ok(BindingValue::Vector3(x)),
        _ => Err(EvaluationError {
            message: format!("bad postfix: there should be a value, but there isn't"),
        }),
    }
}

pub fn precompile_expression(expression: &str) -> Result<PrecompiledCode, CompilationError> {
    // TODO: provide proper expression error messages
    let (tokenized, binds) = tokenize(expression).expect("error in expression");
    let postfix = to_postfix(tokenized).expect("expressin syntax error");
    if let Err(error) = validate_postfix(&postfix) {
        return Err(error);
    }
    Ok(PrecompiledCode {
        postfix: postfix,
        bindings: binds        
    })
}

pub fn evaluate_expression_precompiled(
    precomp: &PrecompiledCode,
    binding_value_map: &HashMap<String, BindingValue>,
) -> Result<BindingValue, EvaluationError> {
    let bindings = precomp.binding_map_to_values(binding_value_map);
    evaluate_postfix(&precomp.postfix, &bindings)
}

pub fn evaluate_expression_precompiled_with_bindings(
    precomp: &PrecompiledCode,
    bindings: &Vec<BindingValue>,
) -> Result<BindingValue, EvaluationError> {
    evaluate_postfix(&precomp.postfix, bindings)
}

pub fn evaluate_expression(expression: &str, binding_value_map: &HashMap<String, BindingValue>) -> Result<BindingValue, ExpressionError> {
    let precomp = match precompile_expression(expression) {
        Ok(x) => x,
        Err(e) => { return Err(ExpressionError::CompilationError(e)); }
    };
    match evaluate_expression_precompiled(&precomp, binding_value_map) {
        Ok(x) => Ok(x),
        Err(e) => Err(ExpressionError::EvaluationError(e))
    }
}

///
/// --------------------------------------------------
///                       TESTS
/// --------------------------------------------------
/// 

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! compare_tokens {
        ($test:expr, $($token:expr),*) => {{
            let expected = vec![$($token),*];
            assert_eq!($test.len(), expected.len());
            for (i, (tok_act, tok_exp)) in $test.iter().zip(&expected).enumerate() {
                assert_eq!(*tok_act, *tok_exp, "tokens mismatch at num {}", i);
            };
        }};
    }

    macro_rules! test_tokenization {
        ($expr:literal, $($token:expr),*) => {{
            let (tokenized, bindings) = tokenize($expr).expect("tokenization failed");
            
            compare_tokens!(tokenized, $($token),*);
            (tokenized, bindings)
        }}
    }

    macro_rules! test_error_postfix {
        ($expr:literal) => {
            let (tokens, _) = tokenize($expr).expect("tokenization failed");
            match to_postfix(tokens) {
                Err(error) => { println!("Ok: {}", error); }
                Ok(postfix) => { 
                    let error = validate_postfix(&postfix).expect_err(&format!("valudation succeeded, that's a fail! {:?}", postfix));
                    println!("OK: {}", error);
                }
            }  
        };
    }

    macro_rules! test_succ_postfix {
        ($expr:literal) => {
            let (tokens, _) = tokenize($expr).expect("tokenization failed");
            let postfix = to_postfix(tokens).expect(&format!("postfix failed {:?}", $expr));
            validate_postfix(&postfix).expect(&format!("valudation failed {:?}", postfix));
            println!("OK: {}", $expr);
        };
    }

    macro_rules! test_expression_result {
        ($expr:literal, $res:expr) => {
            let precomp = precompile_expression($expr).expect("failed to 'compile'");
            let result = evaluate_expression_precompiled_with_bindings(&precomp, &vec![]).expect("failed to evaluate");
            assert_eq!($res, result);
        };
    }

    #[test]
    fn test_tokenize() {
        test_tokenization!(
            "1 +  2",
            Token::IntValue(1),
            Token::BinaryPlus,
            Token::IntValue(2)
        );

        test_tokenization!(
            "1.1   -  2.4",
            Token::FloatValue(1.1),
            Token::BinaryMinus,
            Token::FloatValue(2.4)
        );

        test_tokenization!(
            "{1, 2.1 ,  5.5}",
            Token::CurlyOpen,
            Token::IntValue(1),
            Token::Comma,
            Token::FloatValue(2.1),
            Token::Comma,
            Token::FloatValue(5.5),
            Token::CurlyClose
        );

        macro_rules! _test_similar1 {
            ($expr:literal, $op:expr) => {
                test_tokenization!(
                    $expr,
                    Token::CurlyOpen,
                    Token::IntValue(1),
                    Token::Comma,
                    Token::IntValue(2),
                    Token::Comma,
                    Token::IntValue(3),
                    Token::CurlyClose,
                    $op,
                    Token::CurlyOpen,
                    Token::IntValue(2),
                    Token::Comma,
                    Token::IntValue(3),
                    Token::Comma,
                    Token::IntValue(4),
                    Token::CurlyClose
                );
            };
        }
        _test_similar1!("{1,2,3}+{2,3,4}", Token::BinaryPlus);
        _test_similar1!("{1,2,3}-{2,3,4}", Token::BinaryMinus);
        _test_similar1!("{1,2,3}*{2,3,4}", Token::Multiply);
        _test_similar1!("{1,2,3}/{2,3,4}", Token::Divide);
        _test_similar1!("{1,2,3} +{2,3,4}", Token::BinaryPlus);
        _test_similar1!("{1,2,3} -{2,3,4}", Token::BinaryMinus);
        _test_similar1!("{1,2,3} *{2,3,4}", Token::Multiply);
        _test_similar1!("{1,2,3} /{2,3,4}", Token::Divide);
        _test_similar1!("{1,2,3}+ {2,3,4}", Token::BinaryPlus);
        _test_similar1!("{1,2,3}- {2,3,4}", Token::BinaryMinus);
        _test_similar1!("{1,2,3}* {2,3,4}", Token::Multiply);
        _test_similar1!("{1,2,3}/ {2,3,4}", Token::Divide);
        
        test_tokenization!(
            "}.x",
            Token::CurlyClose,
            Token::VecIndex(0)
        );

        test_tokenization!(
            "@foo.y",
            Token::Binding(0),
            Token::VecIndex(1)
        );

        test_tokenization!(
            "(2).y",
            Token::BracketOpen,
            Token::IntValue(2),
            Token::BracketClose,
            Token::VecIndex(1)
        );

        test_tokenization!(
            ".2+.1",
            Token::FloatValue(0.2),
            Token::BinaryPlus,
            Token::FloatValue(0.1)
        );

        test_tokenization!(
            "-.2+.1",
            Token::UnaryMinus,
            Token::FloatValue(0.2),
            Token::BinaryPlus,
            Token::FloatValue(0.1)
        );
        test_tokenization!(
            "-2.+1.",
            Token::UnaryMinus,
            Token::FloatValue(2.),
            Token::BinaryPlus,
            Token::FloatValue(1.)
        );

        test_tokenization!(
            "1.2",
            Token::FloatValue(1.2)
        );

        test_tokenization!(
            "{1,2,3}.z + {1,2,3}.y",
            Token::CurlyOpen,
            Token::IntValue(1),
            Token::Comma,
            Token::IntValue(2),
            Token::Comma,
            Token::IntValue(3),
            Token::CurlyClose,
            Token::VecIndex(2),
            Token::BinaryPlus,
            Token::CurlyOpen,
            Token::IntValue(1),
            Token::Comma,
            Token::IntValue(2),
            Token::Comma,
            Token::IntValue(3),
            Token::CurlyClose,
            Token::VecIndex(1)
        );
    }

    #[test]
    fn test_postfix() {
        let (tokens, _) = tokenize("{1, 2.1 ,  5.5}").expect("tokenization failed");
        let postfix = to_postfix(tokens).expect("postfix failed");
        
        println!("{:?}", postfix);
        compare_tokens!(
            postfix,
            Token::IntValue(1),
            Token::FloatValue(2.1),
            Token::FloatValue(5.5),
            Token::MakeVec(3)
        );
        validate_postfix(&postfix).expect(&format!("validation failed for {:?}", postfix));

        let (tokens, _) = tokenize("-11+{1+2, 2.1 + 3*(2+4)*2 ,  5.5-(1+3)*(4+5)}*9").expect("tokenization failed");
        let postfix = to_postfix(tokens).expect("postfix failed");
        
        println!("{:?}", postfix);
        compare_tokens!(
            postfix,
            Token::IntValue(11),
            Token::UnaryMinus,

            Token::IntValue(1),
            Token::IntValue(2),
            Token::BinaryPlus,

            Token::FloatValue(2.1),
            Token::IntValue(3),
            Token::IntValue(2),
            Token::IntValue(4),
            Token::BinaryPlus,
            Token::Multiply,
            Token::IntValue(2),
            Token::Multiply,
            Token::BinaryPlus,

            Token::FloatValue(5.5),
            Token::IntValue(1),
            Token::IntValue(3),
            Token::BinaryPlus,
            Token::IntValue(4),
            Token::IntValue(5),
            Token::BinaryPlus,
            Token::Multiply,
            Token::BinaryMinus,

            Token::MakeVec(3),

            Token::IntValue(9),
            Token::Multiply,
            Token::BinaryPlus
        );
        validate_postfix(&postfix).expect(&format!("validation failed for {:?}", postfix));

    }

    #[test]
    fn test_postfix_fail() {
        test_error_postfix!("1+");
        test_error_postfix!("1*");
        test_error_postfix!("1+*2");
        test_error_postfix!("1+(2-");
        test_error_postfix!("1+(2-)");
        test_error_postfix!("1+((2-3)");
        test_error_postfix!("1+(2-3))");
        test_error_postfix!("*(1+(2-3)");
        test_error_postfix!("1,2.1 ,5.5}");
        test_error_postfix!("{{1,2.1 ,5.5}");
        test_error_postfix!("1,2.1 ,5.5}");
    }

    #[test]
    fn test_postfix_valid() {
        test_succ_postfix!("1");
        test_succ_postfix!("1+2");
        test_succ_postfix!("1+(2)");
        test_succ_postfix!("(1)+2");
        test_succ_postfix!("@foo");
        test_succ_postfix!("@foo*2");
        test_succ_postfix!("-@foo");
        test_succ_postfix!("1+@foo");
        test_succ_postfix!("@foo+2");
        test_succ_postfix!("@foo*@bar");
        test_succ_postfix!("@foo+1*@bar");
        test_succ_postfix!("{1,2,3}");
        test_succ_postfix!("{1,2,3} + {2,3,4}");
    }

    #[test]
    fn test_expressions() {
        test_expression_result!("1+3", BindingValue::Int(4));
        test_expression_result!("1.0+3", BindingValue::Float(4.));
        test_expression_result!("1+3.0", BindingValue::Float(4.));
        test_expression_result!("1.0+3.0", BindingValue::Float(4.));
        test_expression_result!("{1.0, 2, 3} + {10, 20.0, 30.0}", BindingValue::Vector3(Vector::new(11., 22., 33.)));
        test_expression_result!("{1.0, 2, 3} - {10, 20.0, 30.0}", BindingValue::Vector3(Vector::new(-9., -18., -27.)));
        test_expression_result!("1 + {10, 20.0, 30.0}", BindingValue::Vector3(Vector::new(11., 21., 31.)));
        test_expression_result!("{10, 20.0, 30.0}+1", BindingValue::Vector3(Vector::new(11., 21., 31.)));
        test_expression_result!("{10, 20.0, 30.0}*2", BindingValue::Vector3(Vector::new(20., 40., 60.)));
        test_expression_result!("{10, 20.0, 30.0}/2", BindingValue::Vector3(Vector::new(5., 10., 15.)));
        test_expression_result!("{1+2*3, 5*(2-4), (1.0+5.5)*2}/2", BindingValue::Vector3(Vector::new(3.5, -5., 6.5)));
        test_expression_result!("-{1,2,3}", BindingValue::Vector3(Vector::new(-1., -2., -3.)));
        test_expression_result!("+{1,2,3}", BindingValue::Vector3(Vector::new(1., 2., 3.)));
        test_expression_result!("{1,2,3}.x", BindingValue::Float(1.));
        test_expression_result!("{1,2,3}.y", BindingValue::Float(2.));
        test_expression_result!("{1,2,3}.z", BindingValue::Float(3.));
        test_expression_result!("({1,2,3} + 5.5).x", BindingValue::Float(6.5));
        test_expression_result!("({1,2,3} + 5.5).y", BindingValue::Float(7.5));
        test_expression_result!("({1,2,3} + 5.5).z + (2*{1,2,3}).y", BindingValue::Float(12.5));
        test_expression_result!("({1,2,3} + 5.5).z + {5,2,7}*{1,2,3}.y", BindingValue::Vector3(Vector::new(18.5, 12.5, 22.5)));
    }

    #[test]
    fn test_evaluate() { // TODO: move this test into the prev one
        let (tokens, _) = tokenize("-11+{1+2, 2.1 + 3*(2+4)*2 ,  5.5-(1+3)*(4+5)}*9").expect("tokenization failed");
        let postfix = to_postfix(tokens).expect("postfix failed");

        let value = evaluate_postfix(&postfix, &vec![]).expect("failed to evaluate");

        assert_eq!(BindingValue::Vector3(Vector::new(16., -11.+(2.1+3.*(2.+4.)*2.)*9., -285.5)), value);
    }

    #[test]
    fn test_parsing_simplest() {
        let (tokenized, _) = test_tokenization!(
            "5+33.33*22",
            Token::IntValue(5),
            Token::BinaryPlus,
            Token::FloatValue(33.33),
            Token::Multiply,
            Token::IntValue(22)
        );

        let postfix = to_postfix(tokenized).expect("postfix failed");
        {
            // check 2
            println!("{:?}", postfix);
            let expected = vec![
                Token::IntValue(5),
                Token::FloatValue(33.33),
                Token::IntValue(22),
                Token::Multiply,
                Token::BinaryPlus,
            ];
            assert_eq!(postfix.len(), expected.len());
            for (i, (tok_act, tok_exp)) in postfix.iter().zip(&expected).enumerate() {
                assert_eq!(*tok_act, *tok_exp, "tokens mismatch at num {}", i);
            }
        }

        let result = evaluate_postfix(&postfix, &vec![]).expect("evaluation failed");
        {
            // check 3
            assert_eq!(result, BindingValue::Float(738.26));
        }
    }
    
    #[test]
    fn test_parsing_variables() {
        let (tokenized, bindings) = test_tokenization!(
            "-2*(3.4+@a)-91*@bc",
            Token::UnaryMinus,
            Token::IntValue(2),
            Token::Multiply,
            Token::BracketOpen,
            Token::FloatValue(3.4),
            Token::BinaryPlus,
            Token::Binding(0),
            Token::BracketClose,
            Token::BinaryMinus,
            Token::IntValue(91),
            Token::Multiply,
            Token::Binding(1)
        );
          

        let postfix = to_postfix(tokenized).expect("postfix failed");
        {
            // check 2
            println!("{:?}", postfix);
            let expected = vec![
                Token::IntValue(2),
                Token::UnaryMinus,
                Token::FloatValue(3.4),
                Token::Binding(0),
                Token::BinaryPlus,
                Token::Multiply,
                Token::IntValue(91),
                Token::Binding(1),
                Token::Multiply,
                Token::BinaryMinus,
            ];
            assert_eq!(postfix.len(), expected.len());
            for (i, (tok_act, tok_exp)) in postfix.iter().zip(&expected).enumerate() {
                assert_eq!(*tok_act, *tok_exp, "tokens mismatch at num {}", i);
            }
        }

        assert_eq!(bindings[0].name, "a");
        assert_eq!(bindings[1].name, "bc");
        let mut bind_values: Vec<BindingValue> = bindings.iter().map(|b| { b.value }).collect();
        bind_values[0] = BindingValue::Float(7.79);
        bind_values[1] = BindingValue::Float(2.345);
        
        let result = evaluate_postfix(&postfix, &bind_values).expect("evaluation failed");
        {
            // check 3
            assert_eq!(result, BindingValue::Float(-235.775));
        }
    }
}
