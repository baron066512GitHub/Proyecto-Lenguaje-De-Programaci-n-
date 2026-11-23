//! Analizador léxico (tokenizador) basado en separación por espacios.
//!
//! Convierte una línea de entrada `&str` en un vector de tokens `Vec<Token>`,
//! validando literales numéricos, booleanos, operadores, comandos e identificadores.

use crate::error::EvalError;
use crate::types::{Command, Operator, Token, Value};

/// Verifica si una cadena cumple las reglas de identificador válido:
/// comienza por letra ASCII o `_`, seguida de caracteres alfanuméricos ASCII o `_`.
pub fn is_valid_identifier(word: &str) -> bool {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        _ => false,
    }
}

/// Clasifica un lexema individual (`&str`) en su `Token` correspondiente.
fn parse_word(word: &str) -> Result<Token, EvalError> {
    // 1. Comandos del REPL y palabras reservadas de alcance
    let token = match word {
        "begin" => Token::Command(Command::Begin),
        "end" => Token::Command(Command::End),
        "vars" => Token::Command(Command::Vars),
        "help" => Token::Command(Command::Help),
        "exit" | "quit" => Token::Command(Command::Quit),

        // 2. Literales booleanos
        "true" => Token::Literal(Value::Bool(true)),
        "false" => Token::Literal(Value::Bool(false)),

        // 3. Operadores aritméticos, relacionales, lógicos y asignación
        "+" => Token::Operator(Operator::Add),
        "-" => Token::Operator(Operator::Sub),
        "*" => Token::Operator(Operator::Mul),
        "/" => Token::Operator(Operator::Div),
        "==" => Token::Operator(Operator::Eq),
        "!=" => Token::Operator(Operator::Ne),
        "<" => Token::Operator(Operator::Lt),
        ">" => Token::Operator(Operator::Gt),
        "<=" => Token::Operator(Operator::Le),
        ">=" => Token::Operator(Operator::Ge),
        "and" => Token::Operator(Operator::And),
        "or" => Token::Operator(Operator::Or),
        "not" => Token::Operator(Operator::Not),
        "=" => Token::Operator(Operator::Assign),

        // 4. Números enteros, flotantes o identificadores
        other => {
            // Intentar parseo como entero de 64 bits usando sintaxis turbofish (`::<i64>`)
            if let Ok(int_val) = other.parse::<i64>() {
                Token::Literal(Value::Int(int_val))
            } else if looks_like_float(other) {
                match other.parse::<f64>() {
                    Ok(float_val) if float_val.is_finite() => {
                        Token::Literal(Value::Float(float_val))
                    }
                    _ => return Err(EvalError::InvalidToken(other.to_string())),
                }
            } else if is_valid_identifier(other) {
                Token::Ident(other.to_string())
            } else {
                return Err(EvalError::InvalidToken(other.to_string()));
            }
        }
    };

    Ok(token)
}

/// Comprueba si un lexema tiene forma numérica decimal (ej. `3.14`, `-0.5`, `1e3`)
/// evitando que palabras como `inf` o `NaN` se interpreten como flotantes.
fn looks_like_float(word: &str) -> bool {
    let stripped = word.strip_prefix('-').or_else(|| word.strip_prefix('+')).unwrap_or(word);
    let has_digit = stripped.bytes().any(|b| b.is_ascii_digit());
    let starts_numeric = stripped
        .bytes()
        .next()
        .is_some_and(|b| b.is_ascii_digit() || b == b'.');
    has_digit && starts_numeric
}

/// Divide la línea de entrada por espacios en blanco y construye la secuencia de tokens.
///
/// Recibe un slice inmutable `&str` (préstamo de solo lectura sin copia) y utiliza
/// `Iterator::collect::<Result<Vec<_>, _>>()` para propagar el primer error léxico.
pub fn tokenize(input: &str) -> Result<Vec<Token>, EvalError> {
    input.split_whitespace().map(parse_word).collect()
}
