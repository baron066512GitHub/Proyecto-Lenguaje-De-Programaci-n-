//! Tipos de datos compuestos (ADTs) del evaluador RPN.
//!
//! Modela valores (`Value`), operadores (`Operator`), comandos (`Command`),
//! tokens (`Token`) y entradas de pila (`StackEntry`) mediante `enum` algebraicos.

use std::fmt;

/// Valores en tiempo de ejecución soportados por el evaluador RPN.
///
/// Implementa `Copy` porque todas sus variantes contienen tipos escalares
/// almacenados íntegramente en la pila (stack), sin propiedad sobre el heap.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl Value {
    /// Retorna el nombre estático del tipo del valor para mensajes y diagnósticos.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Bool(_) => "Bool",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(x) => {
                // Evita mostrar -0.0 y garantiza formato legible con punto decimal
                let normalized = if *x == 0.0 { 0.0 } else { *x };
                if normalized.fract() == 0.0 {
                    write!(f, "{:.1}", normalized)
                } else {
                    write!(f, "{}", normalized)
                }
            }
            Value::Bool(b) => write!(f, "{}", b),
        }
    }
}

/// Operadores aritméticos, relacionales, lógicos y de asignación en RPN.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    // Aritméticos binarios
    Add,
    Sub,
    Mul,
    Div,
    // Comparaciones lógicas binarias
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    // Operadores booleanos
    And,
    Or,
    Not,
    // Asignación de variables en postfija (ej. `x 10 =`)
    Assign,
}

impl Operator {
    /// Devuelve la aridad (número de operandos) requerida por el operador.
    pub fn arity(self) -> usize {
        match self {
            Operator::Not => 1,
            _ => 2,
        }
    }

    /// Devuelve el símbolo textual del operador.
    pub fn symbol(self) -> &'static str {
        match self {
            Operator::Add => "+",
            Operator::Sub => "-",
            Operator::Mul => "*",
            Operator::Div => "/",
            Operator::Eq => "==",
            Operator::Ne => "!=",
            Operator::Lt => "<",
            Operator::Gt => ">",
            Operator::Le => "<=",
            Operator::Ge => ">=",
            Operator::And => "and",
            Operator::Or => "or",
            Operator::Not => "not",
            Operator::Assign => "=",
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.symbol())
    }
}

/// Comandos interactivos del Mini-REPL y control de ámbitos locales.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Abre un nuevo ámbito léxico local (`begin`).
    Begin,
    /// Cierra el ámbito local actual (`end`).
    End,
    /// Muestra todas las variables activas en la tabla de símbolos (`vars`).
    Vars,
    /// Muestra la ayuda rápida de sintaxis y comandos (`help`).
    Help,
    /// Finaliza la sesión del Mini-REPL (`exit` o `quit`).
    Quit,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Command::Begin => "begin",
            Command::End => "end",
            Command::Vars => "vars",
            Command::Help => "help",
            Command::Quit => "quit",
        };
        f.write_str(label)
    }
}

/// Representación léxica de cada elemento separado por espacios en la entrada.
///
/// A diferencia de `Value` y `Operator`, `Token` **no** implementa `Copy`
/// porque la variante `Ident(String)` posee memoria asignada en el heap.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Literal(Value),
    Operator(Operator),
    Command(Command),
    Ident(String),
}

/// Elemento almacenado en la pila de evaluación.
///
/// Diferenciar `Val(Value)` de `Ident(String)` permite evaluar asignaciones
/// postfijas estrictas como `x 10 =`, `x 5 2 + =` o `x x 1 + =`, resolviendo
/// los identificadores perezosamente cuando se usan como operandos de lectura.
#[derive(Debug, Clone, PartialEq)]
pub enum StackEntry {
    Val(Value),
    Ident(String),
}

impl fmt::Display for StackEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StackEntry::Val(v) => write!(f, "{} ({})", v, v.type_name()),
            StackEntry::Ident(name) => write!(f, "identificador '{}'", name),
        }
    }
}
