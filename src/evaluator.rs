//! Motor de evaluación postfija (RPN) con chequeo de tipos y paso explícito de parámetros.
//!
//! Demuestra:
//! - Paso por préstamo mutable (`&mut EvalStack`, `&mut S`), préstamo inmutable (`&S`, `&[Token]`)
//!   y paso por valor/copia (`Operator`, `Value`, `StackEntry`).
//! - Despacho estático mediante traits (`Evaluable` y `<S: SymbolTable>`).
//! - Manejo de errores idiomático con `Result` y el operador de propagación `?`.

use crate::environment::SymbolTable;
use crate::error::EvalError;
use crate::lexer::tokenize;
use crate::stack::EvalStack;
use crate::types::{Command, Operator, StackEntry, Token, Value};

/// Resultado estructurado tras evaluar una línea completa en el Mini-REPL.
#[derive(Debug, Clone, PartialEq)]
pub enum EvalOutcome {
    /// Línea vacía o comentario; no produce salida.
    Empty,
    /// Se evaluó una expresión dejando un valor único en el tope de la pila.
    Value(Value),
    /// Se dejaron múltiples valores en la pila tras la evaluación.
    MultipleValues(Vec<Value>),
    /// Se ejecutó una o más acciones/comandos con mensajes informativos (asignaciones, begin/end, vars).
    Messages(Vec<String>),
    /// El usuario solicitó salir del REPL (`exit` / `quit`).
    Exit,
}

/// Trait que abstrae cualquier unidad sintáctica evaluable sobre la pila y el entorno.
pub trait Evaluable {
    fn eval<S: SymbolTable>(
        &self,
        stack: &mut EvalStack,
        env: &mut S,
        messages: &mut Vec<String>,
    ) -> Result<bool, EvalError>;
}

/// Resuelve una entrada de la pila a un `Value` concreto.
///
/// Obsérvese el contraste en los mecanismos de paso de parámetros:
/// - `entry` se recibe **por valor (move)** porque se consume al evaluarse.
/// - `env` se recibe por **préstamo inmutable (`&S`)** porque solo consulta variables.
pub fn resolve_entry<S: SymbolTable>(entry: StackEntry, env: &S) -> Result<Value, EvalError> {
    match entry {
        StackEntry::Val(v) => Ok(v),
        StackEntry::Ident(name) => env
            .lookup(&name)
            .ok_or(EvalError::UndefinedVariable(name)),
    }
}

/// Aplica un operador aritmético binario (`+`, `-`, `*`, `/`) con chequeo estricto de tipos.
///
/// Reglas de tipos del evaluador:
/// - `Int` op `Int` -> `Int` (aritmética verificada contra overflow y división por cero).
/// - `Int` op `Float` | `Float` op `Int` | `Float` op `Float` -> `Float` (promoción explícita `as f64`).
/// - Cualquier operando `Bool` produce `EvalError::TypeMismatch`.
fn eval_arithmetic(op: Operator, lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => {
            let result = match op {
                Operator::Add => a.checked_add(b),
                Operator::Sub => a.checked_sub(b),
                Operator::Mul => a.checked_mul(b),
                Operator::Div => {
                    if b == 0 {
                        return Err(EvalError::DivisionByZero);
                    }
                    a.checked_div(b)
                }
                _ => unreachable!("eval_arithmetic solo recibe operadores aritméticos"),
            };
            result
                .map(Value::Int)
                .ok_or_else(|| EvalError::IntegerOverflow {
                    operation: op.symbol().to_string(),
                })
        }
        (Value::Float(a), Value::Float(b)) => eval_float_op(op, a, b),
        (Value::Int(a), Value::Float(b)) => eval_float_op(op, a as f64, b),
        (Value::Float(a), Value::Int(b)) => eval_float_op(op, a, b as f64),
        (a, b) => Err(EvalError::TypeMismatch {
            operation: op.symbol().to_string(),
            expected: "operandos numéricos (Int o Float)",
            found: format!("({}, {})", a.type_name(), b.type_name()),
        }),
    }
}

/// Ejecuta una operación aritmética sobre operandos `f64`, verificando división por cero.
fn eval_float_op(op: Operator, a: f64, b: f64) -> Result<Value, EvalError> {
    let result = match op {
        Operator::Add => a + b,
        Operator::Sub => a - b,
        Operator::Mul => a * b,
        Operator::Div => {
            if b == 0.0 {
                return Err(EvalError::DivisionByZero);
            }
            a / b
        }
        _ => unreachable!("eval_float_op solo recibe operadores aritméticos"),
    };
    Ok(Value::Float(result))
}

/// Aplica operadores de comparación relacional (`<`, `>`, `<=`, `>=`) sobre números.
fn eval_ordering(op: Operator, lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    let cmp_result = match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => match op {
            Operator::Lt => a < b,
            Operator::Gt => a > b,
            Operator::Le => a <= b,
            Operator::Ge => a >= b,
            _ => unreachable!(),
        },
        (Value::Float(a), Value::Float(b)) => compare_floats(op, a, b),
        (Value::Int(a), Value::Float(b)) => compare_floats(op, a as f64, b),
        (Value::Float(a), Value::Int(b)) => compare_floats(op, a, b as f64),
        (a, b) => {
            return Err(EvalError::TypeMismatch {
                operation: op.symbol().to_string(),
                expected: "operandos numéricos (Int o Float)",
                found: format!("({}, {})", a.type_name(), b.type_name()),
            })
        }
    };
    Ok(Value::Bool(cmp_result))
}

fn compare_floats(op: Operator, a: f64, b: f64) -> bool {
    match op {
        Operator::Lt => a < b,
        Operator::Gt => a > b,
        Operator::Le => a <= b,
        Operator::Ge => a >= b,
        _ => unreachable!(),
    }
}

/// Aplica comparaciones de igualdad (`==`, `!=`) entre tipos compatibles.
///
/// Permite comparar números entre sí (`Int`/`Float`) y booleanos entre sí (`Bool`/`Bool`),
/// pero rechaza comparaciones heterogéneas número-booleano como `5 true ==`.
fn eval_equality(op: Operator, lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    let is_equal = match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::Int(a), Value::Float(b)) => (a as f64) == b,
        (Value::Float(a), Value::Int(b)) => a == (b as f64),
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (a, b) => {
            return Err(EvalError::TypeMismatch {
                operation: op.symbol().to_string(),
                expected: "operandos del mismo dominio (ambos numéricos o ambos Bool)",
                found: format!("({}, {})", a.type_name(), b.type_name()),
            })
        }
    };

    let result = match op {
        Operator::Eq => is_equal,
        Operator::Ne => !is_equal,
        _ => unreachable!(),
    };
    Ok(Value::Bool(result))
}

/// Aplica operadores lógicos binarios (`and`, `or`) exclusivamente sobre `Bool`.
fn eval_logical_binary(op: Operator, lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Bool(a), Value::Bool(b)) => {
            let result = match op {
                Operator::And => a && b,
                Operator::Or => a || b,
                _ => unreachable!(),
            };
            Ok(Value::Bool(result))
        }
        (a, b) => Err(EvalError::TypeMismatch {
            operation: op.symbol().to_string(),
            expected: "operandos booleanos (Bool, Bool)",
            found: format!("({}, {})", a.type_name(), b.type_name()),
        }),
    }
}

/// Ejecuta una asignación en notación postfija (`<ident> <expr> =`).
///
/// Recibe `stack: &mut EvalStack` y `env: &mut S` explícitamente por préstamo mutable
/// para extraer el destino y valor de la pila y registrar el enlace en el ámbito activo.
pub fn apply_assignment<S: SymbolTable>(
    stack: &mut EvalStack,
    env: &mut S,
) -> Result<(String, Value), EvalError> {
    let (lhs_entry, rhs_entry) = stack.pop_two("=")?;
    let value = resolve_entry(rhs_entry, env)?;

    match lhs_entry {
        StackEntry::Ident(var_name) => {
            env.define(var_name.clone(), value);
            Ok((var_name, value))
        }
        other => Err(EvalError::InvalidAssignmentTarget(other.to_string())),
    }
}

/// Ejecuta cualquier operador RPN modificando explícitamente la pila y el entorno.
pub fn apply_operator<S: SymbolTable>(
    op: Operator,
    stack: &mut EvalStack,
    env: &mut S,
    messages: &mut Vec<String>,
) -> Result<(), EvalError> {
    match op {
        Operator::Assign => {
            let (name, value) = apply_assignment(stack, env)?;
            messages.push(format!("{} = {} : {}", name, value, value.type_name()));
            Ok(())
        }
        Operator::Not => {
            let operand_entry = stack.pop(op.symbol())?;
            let operand = resolve_entry(operand_entry, env)?;
            match operand {
                Value::Bool(b) => {
                    stack.push(StackEntry::Val(Value::Bool(!b)));
                    Ok(())
                }
                other => Err(EvalError::TypeMismatch {
                    operation: op.symbol().to_string(),
                    expected: "un operando booleano (Bool)",
                    found: other.type_name().to_string(),
                }),
            }
        }
        Operator::Add | Operator::Sub | Operator::Mul | Operator::Div => {
            let (lhs_entry, rhs_entry) = stack.pop_two(op.symbol())?;
            let lhs = resolve_entry(lhs_entry, env)?;
            let rhs = resolve_entry(rhs_entry, env)?;
            let result = eval_arithmetic(op, lhs, rhs)?;
            stack.push(StackEntry::Val(result));
            Ok(())
        }
        Operator::Lt | Operator::Gt | Operator::Le | Operator::Ge => {
            let (lhs_entry, rhs_entry) = stack.pop_two(op.symbol())?;
            let lhs = resolve_entry(lhs_entry, env)?;
            let rhs = resolve_entry(rhs_entry, env)?;
            let result = eval_ordering(op, lhs, rhs)?;
            stack.push(StackEntry::Val(result));
            Ok(())
        }
        Operator::Eq | Operator::Ne => {
            let (lhs_entry, rhs_entry) = stack.pop_two(op.symbol())?;
            let lhs = resolve_entry(lhs_entry, env)?;
            let rhs = resolve_entry(rhs_entry, env)?;
            let result = eval_equality(op, lhs, rhs)?;
            stack.push(StackEntry::Val(result));
            Ok(())
        }
        Operator::And | Operator::Or => {
            let (lhs_entry, rhs_entry) = stack.pop_two(op.symbol())?;
            let lhs = resolve_entry(lhs_entry, env)?;
            let rhs = resolve_entry(rhs_entry, env)?;
            let result = eval_logical_binary(op, lhs, rhs)?;
            stack.push(StackEntry::Val(result));
            Ok(())
        }
    }
}

impl Evaluable for Token {
    /// Evalúa un token individual sobre la pila y el entorno.
    /// Retorna `Ok(true)` si el token solicita terminar el REPL (`Quit`), `Ok(false)` en caso contrario.
    fn eval<S: SymbolTable>(
        &self,
        stack: &mut EvalStack,
        env: &mut S,
        messages: &mut Vec<String>,
    ) -> Result<bool, EvalError> {
        match self {
            Token::Literal(value) => {
                // `Value` es `Copy`: `*value` realiza una copia barata en el stack.
                stack.push(StackEntry::Val(*value));
                Ok(false)
            }
            Token::Ident(name) => {
                // `String` no es `Copy`: se clona explícitamente para transferirlo a la pila.
                stack.push(StackEntry::Ident(name.clone()));
                Ok(false)
            }
            Token::Operator(op) => {
                apply_operator(*op, stack, env, messages)?;
                Ok(false)
            }
            Token::Command(cmd) => match cmd {
                Command::Begin => {
                    env.enter_scope();
                    messages.push(format!(
                        "Bloque local abierto (nivel de alcance: {})",
                        env.depth()
                    ));
                    Ok(false)
                }
                Command::End => {
                    env.exit_scope()?;
                    let level = env.depth();
                    let label = if level == 0 {
                        String::from("global")
                    } else {
                        format!("local:{}", level)
                    };
                    messages.push(format!("Bloque local cerrado (retorno a ámbito {})", label));
                    Ok(false)
                }
                Command::Vars => {
                    messages.push(env.format_variables());
                    Ok(false)
                }
                Command::Help => {
                    messages.push(help_text().to_string());
                    Ok(false)
                }
                Command::Quit => Ok(true),
            },
        }
    }
}

/// Evalúa una secuencia inmutable de tokens (`&[Token]`) usando explícitamente
/// la pila (`&mut EvalStack`) y la tabla de símbolos (`&mut S`).
pub fn evaluate_tokens<S: SymbolTable>(
    tokens: &[Token],
    stack: &mut EvalStack,
    env: &mut S,
) -> Result<EvalOutcome, EvalError> {
    if tokens.is_empty() {
        return Ok(EvalOutcome::Empty);
    }

    stack.clear();
    let mut messages = Vec::new();

    for token in tokens {
        let should_exit = token.eval(stack, env, &mut messages)?;
        if should_exit {
            return Ok(EvalOutcome::Exit);
        }
    }

    // Resolvemos los elementos remanentes en la pila (por ejemplo, consultar `x`).
    if stack.is_empty() {
        return Ok(EvalOutcome::Messages(messages));
    }

    let mut resolved = Vec::with_capacity(stack.len());
    for entry in stack.as_slice() {
        resolved.push(resolve_entry(entry.clone(), env)?);
    }

    if resolved.len() == 1 && messages.is_empty() {
        Ok(EvalOutcome::Value(resolved[0]))
    } else if messages.is_empty() {
        Ok(EvalOutcome::MultipleValues(resolved))
    } else {
        // Si hubo mensajes (ej. `begin x 10 = x 2 * end`) y también un resultado final
        let formatted: Vec<String> = resolved
            .iter()
            .map(|v| format!("=> {} : {}", v, v.type_name()))
            .collect();
        messages.extend(formatted);
        Ok(EvalOutcome::Messages(messages))
    }
}

/// Punto de entrada de alto nivel para evaluar una línea de texto en RPN.
pub fn evaluate_line<S: SymbolTable>(
    line: &str,
    stack: &mut EvalStack,
    env: &mut S,
) -> Result<EvalOutcome, EvalError> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(EvalOutcome::Empty);
    }

    let tokens = tokenize(trimmed)?;
    evaluate_tokens(&tokens, stack, env)
}

/// Texto de ayuda interactiva para el comando `help`.
pub fn help_text() -> &'static str {
    concat!(
        "Comandos y operadores disponibles (Notación Polaca Inversa / RPN):\n",
        "  Aritméticos  : +  -  *  /          (ej: 10 5 + 2 *  -> 30)\n",
        "  Relacionales : ==  !=  <  >  <= >= (ej: 10 5 >      -> true)\n",
        "  Lógicos      : and  or  not        (ej: true false and -> false)\n",
        "  Variables    : <id> <expr> =       (ej: x 10 =  |  x 5 2 + =)\n",
        "  Ámbito local : begin / end         (crea/cierra bloque con shadowing)\n",
        "  Inspección   : vars                (lista variables por ámbito)\n",
        "  Utilidades   : help  |  exit/quit  (ayuda / salir)"
    )
}
