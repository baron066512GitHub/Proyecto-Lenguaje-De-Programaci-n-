//! Pila de evaluación encapsulada para expresiones en Notación Polaca Inversa.
//!
//! Encapsula un `Vec<StackEntry>` privado para garantizar invariantes de acceso
//! (LIFO) y reportar errores de subdesbordamiento (*stack underflow*) mediante `Result`.

use crate::error::EvalError;
use crate::types::StackEntry;

/// Estructura de pila encapsulada.
///
/// El campo `items` es privado: solo puede manipularse mediante los métodos
/// públicos del bloque `impl`, demostrando abstracción de datos y encapsulamiento.
#[derive(Debug, Default, Clone)]
pub struct EvalStack {
    items: Vec<StackEntry>,
}

impl EvalStack {
    /// Crea una pila de evaluación vacía.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Inserta un elemento en el tope de la pila, tomando su propiedad (*move*).
    pub fn push(&mut self, entry: StackEntry) {
        self.items.push(entry);
    }

    /// Extrae un elemento del tope o retorna `EvalError::StackUnderflow` si está vacía.
    pub fn pop(&mut self, operation: &str) -> Result<StackEntry, EvalError> {
        self.items.pop().ok_or_else(|| EvalError::StackUnderflow {
            operation: operation.to_string(),
            expected: 1,
            found: 0,
        })
    }

    /// Extrae dos operandos `(lhs, rhs)` en orden de evaluación postfija.
    ///
    /// Valida la longitud antes de extraer para evitar mutaciones parciales en caso de error.
    pub fn pop_two(&mut self, operation: &str) -> Result<(StackEntry, StackEntry), EvalError> {
        if self.items.len() < 2 {
            return Err(EvalError::StackUnderflow {
                operation: operation.to_string(),
                expected: 2,
                found: self.items.len(),
            });
        }
        // Invariante garantizada por la comprobación previa: existen al menos 2 elementos.
        let rhs = self.items.pop().expect("longitud verificada >= 2");
        let lhs = self.items.pop().expect("longitud verificada >= 2");
        Ok((lhs, rhs))
    }

    /// Vacía la pila reutilizando la capacidad ya reservada en el heap.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Retorna el número de elementos actualmente en la pila.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Indica si la pila está vacía.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Expone una vista inmutable (`&[StackEntry]`) sin transferir propiedad ni copiar datos.
    pub fn as_slice(&self) -> &[StackEntry] {
        &self.items
    }
}
