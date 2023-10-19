use std::collections::HashMap;
use std::fmt;
use super::vec_types::Vector;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Token {
    Binding(usize),
    IntValue(i64),
    FloatValue(f64),
    VectorValue(Vector<f64, 3>),
    BracketOpen,
    BracketClose,
    BinaryPlus,
    UnaryPlus,
    BinaryMinus,
    UnaryMinus,
    Multiply,
    Divide,
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
            Token::BinaryPlus => write!(f, "[binary+]"),
            Token::UnaryPlus => write!(f, "[unary+]"),
            Token::BinaryMinus => write!(f, "[binary-]"),
            Token::UnaryMinus => write!(f, "[unary-]"),
            Token::Multiply => write!(f, "[*]"),
            Token::Divide => write!(f, "[/]"),
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

pub struct Error {
    message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<ERROR HAPPENED: {}>", self.message)
    }
}

#[derive(Copy, Clone)]
enum ValueType {
    NotInitialized,
    Int,
    Float,
    Binding,
}

/// parses human readable expression into a token array
/// returns token vector, and a map of binding nums to binding names
fn tokenize(line: &str) -> Result<(Vec<Token>, Vec<Binding>), Error> {
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
                ValueType::NotInitialized => {
                    // nothing, no token started
                }
            }
            isval = ValueType::NotInitialized;
        };
    }

    for (i, c) in line.trim().chars().enumerate() {
        match (isval, c) {
            (_, c) if c.is_ascii_whitespace() => {
                finalize_token_if_started!();
            }
            (ValueType::NotInitialized | ValueType::Float | ValueType::Int, '0'..='9' | '.') => {
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
            (ValueType::NotInitialized, '@') => {
                isval = ValueType::Binding;
                token_start = i + 1;
            }
            (ValueType::Binding, c) if c.is_ascii_alphanumeric() => {
                token_end = i;
            }
            (_, c) => {
                // first finalize any numeric token if any started
                if let ValueType::NotInitialized = isval {
                } else {
                    finalize_token_if_started!();
                }

                line_tokens.push(match c {
                    '+' => match line_tokens.last() {
                        Some(Token::IntValue(_) | Token::FloatValue(_) | Token::Binding(_) | Token::BracketClose) => {
                            Token::BinaryPlus
                        }
                        _ => Token::UnaryPlus,
                    },
                    '-' => match line_tokens.last() {
                        Some(Token::IntValue(_) | Token::FloatValue(_) | Token::Binding(_) | Token::BracketClose) => {
                            Token::BinaryMinus
                        }
                        _ => Token::UnaryMinus,
                    },
                    '*' => Token::Multiply,
                    '/' => Token::Divide,
                    '(' => Token::BracketOpen,
                    ')' => Token::BracketClose,
                    _ => {
                        return Err(Error {
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

fn to_postfix(tokens_sequence: Vec<Token>) -> Result<Vec<Token>, Error> {
    let mut stack: Vec<Token> = Vec::new();
    let mut result: Vec<Token> = Vec::new();
    for token in tokens_sequence {
        // println!("{:?} [[{:?}", token, stack);
        match token {
            Token::IntValue(_) | Token::FloatValue(_) | Token::VectorValue(_) | Token::Binding(_) => result.push(token),
            Token::UnaryPlus | Token::UnaryMinus => stack.push(token),
            Token::BracketOpen => stack.push(Token::BracketOpen),
            Token::BracketClose => loop {
                match stack.pop() {
                    Some(Token::BracketOpen) => break,
                    Some(x) => result.push(x),
                    None => {
                        return Err(Error {
                            message: format!("bracket mismatch!"),
                        })
                    }
                };
            },
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
fn evaluate_postfix(postfix_sequence: &Vec<Token>, bindings: &Vec<BindingValue>) -> Result<BindingValue, Error> {
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
                    x @ BindingValue::Unknown => {
                        return Err(Error {
                            message: format!("binding {:?} is not set", b),
                            // TODO: error should provide binding num so that wrapping func may
                            //  present a better formed error message
                        });
                    }
                    _ => {
                        panic!("NOT IMPLEMENTED YET !!");
                    }
                }
            }
            Token::UnaryPlus => {
                match stack.pop() {
                    Some(Token::IntValue(x)) => stack.push(Token::IntValue(x)),
                    Some(Token::FloatValue(x)) => stack.push(Token::FloatValue(x)),
                    _ => {
                        return Err(Error {
                            message: format!("bad postfix: misplaced unary plus"),
                        })
                    }
                };
            }
            Token::UnaryMinus => {
                match stack.pop() {
                    Some(Token::IntValue(x)) => stack.push(Token::IntValue(-x)),
                    Some(Token::FloatValue(x)) => stack.push(Token::FloatValue(-x)),
                    x => {
                        return Err(Error {
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
                                return Err(Error {
                                    message: format!(
                                        "bad expression: operation {} is not defined for arguments {} and {}",
                                        token,
                                        x,
                                        y
                                    )
                                })
                            },
                            _ => {
                                return Err(Error {
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
                    VectorValue, IntValue, VectorValue, Vector<f64, 3>, f64,
                    // now swapped arg ops
                    IntValue, VectorValue, VectorValue, f64, Vector<f64, 3> |
                    FloatValue, VectorValue, VectorValue, f64, Vector<f64, 3>
                );

                if !matched {
                    return Err(Error {
                        message: format!("cannot perform {:?} for types {:?} and {:?}", token, val1, val2)
                    });
                }
            }
            _ => panic!("there should not be brackets in polish postfix notaion. maybe use a special token type?"),
        }
    }
    match stack.pop() {
        Some(Token::IntValue(x)) => Ok(BindingValue::Int(x)),
        Some(Token::FloatValue(x)) => Ok(BindingValue::Float(x)),
        Some(Token::VectorValue(x)) => Ok(BindingValue::Vector3(x)),
        _ => Err(Error {
            message: format!("bad postfix: there should be a value, but there isn't"),
        }),
    }
}

pub fn precompile_expression(expression: &str) -> PrecompiledCode {
    // TODO: provide proper expression error messages
    let (tokenized, binds) = tokenize(expression).expect("error in expression");
    PrecompiledCode {
        postfix: to_postfix(tokenized).expect("expressin syntax error"),
        bindings: binds        
    }
}

pub fn evaluate_expression_precompiled(
    precomp: &PrecompiledCode,
    binding_value_map: &HashMap<String, BindingValue>,
) -> Result<BindingValue, Error> {
    let bindings = precomp.binding_map_to_values(binding_value_map);
    evaluate_postfix(&precomp.postfix, &bindings)
}

pub fn evaluate_expression_precompiled_with_bindings(
    precomp: &PrecompiledCode,
    bindings: &Vec<BindingValue>,
) -> Result<BindingValue, Error> {
    evaluate_postfix(&precomp.postfix, bindings)
}

pub fn evaluate_expression(expression: &str, binding_value_map: &HashMap<String, BindingValue>) -> Result<BindingValue, Error> {
    let precomp = precompile_expression(expression);
    evaluate_expression_precompiled(&precomp, binding_value_map)
}

/// --------------------------------------------------
///                       TESTS
/// --------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_simplest() {
        let expr = "5+33.33*22";
        let (tokenized, bindings) = tokenize(expr).expect("tokenization failed");

        {
            // check 1
            println!("{:?}", tokenized);
            let expected = vec![
                Token::IntValue(5),
                Token::BinaryPlus,
                Token::FloatValue(33.33),
                Token::Multiply,
                Token::IntValue(22),
            ];
            assert_eq!(tokenized.len(), expected.len());
            for (i, (tok_act, tok_exp)) in tokenized.iter().zip(&expected).enumerate() {
                assert_eq!(*tok_act, *tok_exp, "tokens mismatch at num {}", i);
            }
        }

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
        let expr = "-2*(3.4+@a)-91*@bc";
        let (tokenized, bindings) = tokenize(expr).expect("tokenization failed");

        {
            // check 1
            println!("{:?}", tokenized);
            let expected = vec![
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
                Token::Binding(1),
            ];
            assert_eq!(tokenized.len(), expected.len());
            for (i, (tok_act, tok_exp)) in tokenized.iter().zip(&expected).enumerate() {
                assert_eq!(*tok_act, *tok_exp, "tokens mismatch at num {}", i);
            }
        }

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
