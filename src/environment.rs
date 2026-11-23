//! Tabla de símbolos (entorno) con soporte para alcance léxico y sombreado (*shadowing*).
//!
//! Define el trait `SymbolTable` (abstracción mediante traits / polimorfismo) y su
//! implementación concreta `Environment`, basada en una pila de mapas `Vec<HashMap<String, Value>>`.

use std::collections::{HashMap, HashSet};

use crate::error::EvalError;
use crate::types::Value;

/// Contrato abstracto para una tabla de símbolos con ámbitos anidados.
///
/// Este trait es *dyn-compatible* (object-safe): puede emplearse tanto con
/// despacho estático (`<S: SymbolTable>`) como con despacho dinámico (`&dyn SymbolTable`).
pub trait SymbolTable {
    /// Declara o actualiza una variable en el ámbito activo actual.
    fn define(&mut self, name: String, value: Value);

    /// Busca una variable desde el ámbito más interno hacia el ámbito global.
    fn lookup(&self, name: &str) -> Option<Value>;

    /// Crea un nuevo ámbito local temporal (`begin`).
    fn enter_scope(&mut self);

    /// Cierra el ámbito local más reciente (`end`), liberando sus variables vía RAII.
    fn exit_scope(&mut self) -> Result<(), EvalError>;

    /// Retorna la profundidad de anidamiento actual (`0` = ámbito global).
    fn depth(&self) -> usize;

    /// Genera una representación legible de las variables activas por ámbito (`vars`).
    fn format_variables(&self) -> String;
}

/// Implementación concreta de la tabla de símbolos basada en composición.
///
/// El índice `0` corresponde siempre al ámbito global; cada `begin` apila un nuevo
/// `HashMap<String, Value>` y cada `end` lo desapila, haciendo que Rust invoque
/// determinísticamente `Drop` sobre el mapa local sin alterar el entorno global.
#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    /// Inicializa el entorno con un único ámbito global vacío.
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable for Environment {
    fn define(&mut self, name: String, value: Value) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, value);
        }
    }

    fn lookup(&self, name: &str) -> Option<Value> {
        // Recorrido inverso: los ámbitos locales sombrean a los externos.
        for scope in self.scopes.iter().rev() {
            if let Some(&val) = scope.get(name) {
                return Some(val);
            }
        }
        None
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) -> Result<(), EvalError> {
        if self.scopes.len() <= 1 {
            return Err(EvalError::NoLocalScopeToClose);
        }
        // Al salir de `pop()`, el `HashMap` devuelto se destruye al finalizar la sentencia (RAII).
        let _discarded_scope = self.scopes.pop();
        Ok(())
    }

    fn depth(&self) -> usize {
        self.scopes.len().saturating_sub(1)
    }

    fn format_variables(&self) -> String {
        let total_vars: usize = self.scopes.iter().map(HashMap::len).sum();
        if total_vars == 0 {
            return String::from("(sin variables declaradas)");
        }

        // Recopilamos qué nombres están sombreados en niveles más profundos
        // para mostrar claramente el efecto de scoping y shadowing.
        let mut shadowed_by_inner: Vec<HashSet<&str>> = vec![HashSet::new(); self.scopes.len()];
        let mut seen_inner: HashSet<&str> = HashSet::new();
        for (idx, scope) in self.scopes.iter().enumerate().rev() {
            shadowed_by_inner[idx] = seen_inner.clone();
            for key in scope.keys() {
                seen_inner.insert(key.as_str());
            }
        }

        let mut lines = Vec::new();
        for (level, scope) in self.scopes.iter().enumerate() {
            let header = if level == 0 {
                String::from("[Ámbito 0 - global]")
            } else {
                format!("[Ámbito {} - local]", level)
            };
            lines.push(header);

            if scope.is_empty() {
                lines.push(String::from("  (vacío)"));
                continue;
            }

            let mut entries: Vec<(&String, &Value)> = scope.iter().collect();
            entries.sort_by_key(|(a, _)| *a);

            for (name, val) in entries {
                let tag = if shadowed_by_inner[level].contains(name.as_str()) {
                    " [sombreada]"
                } else {
                    ""
                };
                lines.push(format!(
                    "  {} = {} : {}{}",
                    name,
                    val,
                    val.type_name(),
                    tag
                ));
            }
        }

        lines.join("\n")
    }
}

/// Función auxiliar que utiliza despacho dinámico (`&dyn SymbolTable`)
/// para inspeccionar el estado de cualquier tabla de símbolos en modo solo lectura.
pub fn describe_scope_state(table: &dyn SymbolTable) -> String {
    let scope_label = match table.depth() {
        0 => String::from("global"),
        d => format!("local:{}", d),
    };
    format!("Entorno [{}]:\n{}", scope_label, table.format_variables())
}
