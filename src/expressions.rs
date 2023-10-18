use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Token {
    Binding(u64),
    IntValue(i64),
    FloatValue(f64),
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
    Vector3([f64; 3]),
}

pub struct Binding {
    pub name: String,
    pub value: BindingValue,
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
        write!(f, "<ASS HAPPENED: {}>", self.message)
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
fn tokenize(line: &str) -> Result<(Vec<Token>, HashMap<u64, Binding>), Error> {
    let mut binding_map = HashMap::new();
    let mut binding_map_cur_id = 0_u64;

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
                    binding_map.insert(
                        binding_map_cur_id,
                        Binding {
                            name: token_name,
                            value: BindingValue::Unknown,
                        },
                    );
                    line_tokens.push(Token::Binding(binding_map_cur_id));
                    binding_map_cur_id += 1;
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

    Ok((line_tokens, binding_map))
}

fn to_postfix(tokens_sequence: Vec<Token>) -> Result<Vec<Token>, Error> {
    let mut stack: Vec<Token> = Vec::new();
    let mut result: Vec<Token> = Vec::new();
    for token in tokens_sequence {
        // println!("{:?} [[{:?}", token, stack);
        match token {
            Token::IntValue(_) | Token::FloatValue(_) | Token::Binding(_) => result.push(token),
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
pub fn evaluate_postfix(postfix_sequence: &Vec<Token>, binding_map: &HashMap<u64, Binding>) -> Result<BindingValue, Error> {
    let mut stack: Vec<Token> = Vec::new();
    for token in postfix_sequence.iter() {
        match token {
            Token::IntValue(_) | Token::FloatValue(_) => stack.push(*token),
            Token::Binding(b) => {
                let binding = binding_map.get(&b).expect("token id not present in token map!");
                match &binding.value {
                    BindingValue::Float(x) => stack.push(Token::FloatValue(*x)),
                    BindingValue::Int(x) => stack.push(Token::IntValue(*x)),
                    x @ BindingValue::Unknown => {
                        return Err(Error {
                            message: format!("binding {:?} is not set", binding.name),
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

                macro_rules! sum_helper {
                    ($opt1:tt, $opt2:tt, $res:tt, $rt1:ty, $rt2:ty) => {
                        match (val1, val2) {
                            (Some(Token::$opt1(x)), Some(Token::$opt2(y))) => {
                                stack.push(Token::$res(match token {
                                    Token::BinaryPlus => x as $rt1 + y as $rt2,
                                    Token::BinaryMinus => x as $rt1 - y as $rt2,
                                    Token::Multiply => x as $rt1 * y as $rt2,
                                    Token::Divide => x as $rt1 / y as $rt2,
                                    _ => panic!("impossibruuuu!!"),
                                }));
                            }
                            (Some(_), Some(_)) => (),
                            _ => {
                                return Err(Error {
                                    message: format!("bad postfix: not enough operands for {}", token),
                                })
                            }
                        };
                    };
                }
                sum_helper!(IntValue, IntValue, IntValue, i64, i64);
                sum_helper!(IntValue, FloatValue, FloatValue, f64, f64);
                sum_helper!(FloatValue, IntValue, FloatValue, f64, f64);
                sum_helper!(FloatValue, FloatValue, FloatValue, f64, f64);
            }
            _ => panic!("there should not be brackets in polish postfix notaion. maybe use a special token type?"),
        }
    }
    match stack.pop() {
        Some(Token::IntValue(x)) => Ok(BindingValue::Int(x)),
        Some(Token::FloatValue(x)) => Ok(BindingValue::Float(x)),
        _ => Err(Error {
            message: format!("bad fostfix: there should be a value, but there isn't"),
        }),
    }
}

pub fn precompile_expression(expression: &str) -> (Vec<Token>, HashMap<u64, Binding>) {
    // TODO: provide proper expression error messages
    let (tokenized, bind_map) = tokenize(expression).expect("error in expression");
    (to_postfix(tokenized).expect("expressin syntax error"), bind_map)
}

pub fn evaluate_expression_precompiled(
    postfix: &Vec<Token>,
    binding_map: &mut HashMap<u64, Binding>,
    binding_value_map: &HashMap<String, BindingValue>,
) -> Result<BindingValue, Error> {
    for binding in binding_map.values_mut() {
        if let Some(value) = binding_value_map.get(&binding.name) {
            binding.value = *value;
        }
    }
    evaluate_postfix(postfix, binding_map)
}

pub fn evaluate_expression(expression: &str, binding_value_map: &HashMap<String, BindingValue>) -> Result<BindingValue, Error> {
    let (postfix, mut binding_map) = precompile_expression(expression);
    evaluate_expression_precompiled(&postfix, &mut binding_map, binding_value_map)
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
        let (tokenized, binding_map) = tokenize(expr).expect("tokenization failed");

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

        let result = evaluate_postfix(&postfix, &binding_map).expect("evaluation failed");
        {
            // check 3
            assert_eq!(result, BindingValue::Float(738.26));
        }
    }

    #[test]
    fn test_parsing_variables() {
        let expr = "-2*(3.4+@a)-91*@bc";
        let (tokenized, mut binding_map) = tokenize(expr).expect("tokenization failed");

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

        assert_eq!(binding_map.get_mut(&0).expect("missing binding 0").name, "a");
        assert_eq!(binding_map.get_mut(&1).expect("missing binding 0").name, "bc");
        binding_map.get_mut(&0).expect("missing binding 0").value = BindingValue::Float(7.79);
        binding_map.get_mut(&1).expect("missing binding 0").value = BindingValue::Float(2.345);
        let result = evaluate_postfix(&postfix, &binding_map).expect("evaluation failed");
        {
            // check 3
            assert_eq!(result, BindingValue::Float(-235.775));
        }
    }
}
