//! Definición del tipo de error algebraico del evaluador RPN.
//!
//! Demuestra el manejo idiomático de errores en Rust mediante `enum`,
//! `Result<T, E>` y la implementación de los traits estándar `Display` y `Error`.

use std::error::Error;
use std::fmt;

/// Errores controlados que pueden ocurrir durante el análisis léxico
/// o la evaluación de una expresión RPN.
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    /// Token léxicamente inválido (no es número, booleano, operador, comando ni identificador válido).
    InvalidToken(String),
    /// La pila no contiene suficientes operandos para ejecutar la operación solicitada.
    StackUnderflow {
        operation: String,
        expected: usize,
        found: usize,
    },
    /// Los tipos de los operandos en la pila no son compatibles con el operador.
    TypeMismatch {
        operation: String,
        expected: &'static str,
        found: String,
    },
    /// Intento de división entre cero (entero `0` o flotante `0.0`).
    DivisionByZero,
    /// Desbordamiento aritmético en operaciones con enteros de 64 bits (`i64`).
    IntegerOverflow { operation: String },
    /// Referencia a una variable no declarada en ningún ámbito activo.
    UndefinedVariable(String),
    /// Intento de asignar un valor a algo que no es un identificador (ej. `10 20 =`).
    InvalidAssignmentTarget(String),
    /// Intento de cerrar un bloque (`end`) cuando solo existe el ámbito global.
    NoLocalScopeToClose,
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::InvalidToken(tok) => {
                write!(f, "Error léxico: token inválido '{}'", tok)
            }
            EvalError::StackUnderflow {
                operation,
                expected,
                found,
            } => write!(
                f,
                "Error de pila (stack underflow): '{}' requiere {} operando(s), pero solo hay {} en la pila",
                operation, expected, found
            ),
            EvalError::TypeMismatch {
                operation,
                expected,
                found,
            } => write!(
                f,
                "Error de tipos: '{}' esperaba {}, pero recibió {}",
                operation, expected, found
            ),
            EvalError::DivisionByZero => {
                write!(f, "Error aritmético: división por cero")
            }
            EvalError::IntegerOverflow { operation } => {
                write!(
                    f,
                    "Error aritmético: desbordamiento entero (overflow) en '{}'",
                    operation
                )
            }
            EvalError::UndefinedVariable(name) => {
                write!(
                    f,
                    "Error de nombre: la variable '{}' no está definida en el ámbito actual",
                    name
                )
            }
            EvalError::InvalidAssignmentTarget(found) => write!(
                f,
                "Error de asignación: el destino de '=' debe ser un identificador, pero se encontró {}",
                found
            ),
            EvalError::NoLocalScopeToClose => {
                write!(
                    f,
                    "Error de alcance: no hay ningún bloque local abierto que cerrar con 'end'"
                )
            }
        }
    }
}

impl Error for EvalError {}
