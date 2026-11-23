//! Biblioteca central del Evaluador Postfijo (RPN) y Mini-REPL en Rust.
//!
//! Organiza el sistema en módulos cohesivos que demuestran el diseño e
//! implementación del lenguaje Rust (CAO604 - Universidad de Carabobo).

pub mod environment;
pub mod error;
pub mod evaluator;
pub mod lexer;
pub mod stack;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::environment::{describe_scope_state, Environment, SymbolTable};
    use crate::error::EvalError;
    use crate::evaluator::{evaluate_line, EvalOutcome};
    use crate::stack::EvalStack;
    use crate::types::Value;

    #[test]
    fn arithmetic_and_assignment_from_spec() {
        let mut stack = EvalStack::new();
        let mut env = Environment::new();

        // Ejemplo del enunciado: `x 10 =` y luego `x 5 + 2 *` => (10 + 5) * 2 = 30
        let res1 = evaluate_line("x 10 =", &mut stack, &mut env).unwrap();
        assert_eq!(res1, EvalOutcome::Messages(vec!["x = 10 : Int".to_string()]));

        let res2 = evaluate_line("x 5 + 2 *", &mut stack, &mut env).unwrap();
        assert_eq!(res2, EvalOutcome::Value(Value::Int(30)));

        // Ejemplo del enunciado: `x 5 2 + =` => x = 7
        evaluate_line("x 5 2 + =", &mut stack, &mut env).unwrap();
        assert_eq!(env.lookup("x"), Some(Value::Int(7)));

        // Auto-actualización en RPN: `x x 1 + =` => x = 8
        evaluate_line("x x 1 + =", &mut stack, &mut env).unwrap();
        assert_eq!(env.lookup("x"), Some(Value::Int(8)));
    }

    #[test]
    fn float_promotion_and_comparisons() {
        let mut stack = EvalStack::new();
        let mut env = Environment::new();

        assert_eq!(
            evaluate_line("5 2.5 +", &mut stack, &mut env).unwrap(),
            EvalOutcome::Value(Value::Float(7.5))
        );
        assert_eq!(
            evaluate_line("7.0 2 /", &mut stack, &mut env).unwrap(),
            EvalOutcome::Value(Value::Float(3.5))
        );
        assert_eq!(
            evaluate_line("10 5 >", &mut stack, &mut env).unwrap(),
            EvalOutcome::Value(Value::Bool(true))
        );
        assert_eq!(
            evaluate_line("5 5.0 ==", &mut stack, &mut env).unwrap(),
            EvalOutcome::Value(Value::Bool(true))
        );
        assert_eq!(
            evaluate_line("3 4 !=", &mut stack, &mut env).unwrap(),
            EvalOutcome::Value(Value::Bool(true))
        );
        assert_eq!(
            evaluate_line("true false and not", &mut stack, &mut env).unwrap(),
            EvalOutcome::Value(Value::Bool(true))
        );
    }

    #[test]
    fn controlled_runtime_and_type_errors() {
        let mut stack = EvalStack::new();
        let mut env = Environment::new();

        // Ejemplo del enunciado: `5 true +` -> error de tipos controlado
        assert!(matches!(
            evaluate_line("5 true +", &mut stack, &mut env),
            Err(EvalError::TypeMismatch { .. })
        ));

        // División por cero (entero y flotante)
        assert_eq!(
            evaluate_line("10 0 /", &mut stack, &mut env),
            Err(EvalError::DivisionByZero)
        );
        assert_eq!(
            evaluate_line("10.0 0.0 /", &mut stack, &mut env),
            Err(EvalError::DivisionByZero)
        );

        // Stack underflow
        assert!(matches!(
            evaluate_line("5 +", &mut stack, &mut env),
            Err(EvalError::StackUnderflow {
                expected: 2,
                found: 1,
                ..
            })
        ));

        // Token inválido
        assert_eq!(
            evaluate_line("12abc 3 +", &mut stack, &mut env),
            Err(EvalError::InvalidToken("12abc".to_string()))
        );

        // Variable no definida
        assert_eq!(
            evaluate_line("no_existe 1 +", &mut stack, &mut env),
            Err(EvalError::UndefinedVariable("no_existe".to_string()))
        );

        // Destino de asignación inválido
        assert!(matches!(
            evaluate_line("10 20 =", &mut stack, &mut env),
            Err(EvalError::InvalidAssignmentTarget(_))
        ));

        // Comparación inválida entre Int y Bool
        assert!(matches!(
            evaluate_line("5 true ==", &mut stack, &mut env),
            Err(EvalError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn lexical_scoping_and_shadowing() {
        let mut stack = EvalStack::new();
        let mut env = Environment::new();

        evaluate_line("x 10 =", &mut stack, &mut env).unwrap();
        evaluate_line("y 20 =", &mut stack, &mut env).unwrap();

        // Entrar a ámbito local 1
        evaluate_line("begin", &mut stack, &mut env).unwrap();
        assert_eq!(env.depth(), 1);

        // Puede leer variables externas y sombrear `x` localmente
        assert_eq!(
            evaluate_line("x y +", &mut stack, &mut env).unwrap(),
            EvalOutcome::Value(Value::Int(30))
        );
        evaluate_line("x 99 =", &mut stack, &mut env).unwrap();
        evaluate_line("z 5 =", &mut stack, &mut env).unwrap();

        assert_eq!(
            evaluate_line("x z +", &mut stack, &mut env).unwrap(),
            EvalOutcome::Value(Value::Int(104))
        );

        // Despacho dinámico (&dyn SymbolTable) refleja el sombreado
        let snapshot = describe_scope_state(&env);
        assert!(snapshot.contains("[sombreada]"));
        assert!(snapshot.contains("z = 5 : Int"));

        // Ámbito anidado nivel 2
        evaluate_line("begin", &mut stack, &mut env).unwrap();
        evaluate_line("x 500 =", &mut stack, &mut env).unwrap();
        assert_eq!(env.lookup("x"), Some(Value::Int(500)));

        // Cerrar nivel 2 -> vuelve a x = 99 (nivel 1)
        evaluate_line("end", &mut stack, &mut env).unwrap();
        assert_eq!(env.lookup("x"), Some(Value::Int(99)));

        // Cerrar nivel 1 -> vuelve a global: x = 10, z ya no existe
        evaluate_line("end", &mut stack, &mut env).unwrap();
        assert_eq!(env.depth(), 0);
        assert_eq!(env.lookup("x"), Some(Value::Int(10)));
        assert_eq!(env.lookup("y"), Some(Value::Int(20)));
        assert_eq!(env.lookup("z"), None);

        // Cerrar ámbito global produce error controlado
        assert_eq!(
            evaluate_line("end", &mut stack, &mut env),
            Err(EvalError::NoLocalScopeToClose)
        );
    }

    #[test]
    fn negative_numbers_overflow_and_inline_blocks() {
        let mut stack = EvalStack::new();
        let mut env = Environment::new();

        // Literales negativos y comentarios
        assert_eq!(
            evaluate_line("-10 4 +", &mut stack, &mut env).unwrap(),
            EvalOutcome::Value(Value::Int(-6))
        );
        assert_eq!(
            evaluate_line("   # comentario ignorado", &mut stack, &mut env).unwrap(),
            EvalOutcome::Empty
        );

        // Overflow controlado en i64 (sin panic)
        let overflow_expr = format!("{} 1 +", i64::MAX);
        assert!(matches!(
            evaluate_line(&overflow_expr, &mut stack, &mut env),
            Err(EvalError::IntegerOverflow { .. })
        ));

        // Evaluación con bloque inline: no contamina el ámbito global
        evaluate_line("a 1 =", &mut stack, &mut env).unwrap();
        evaluate_line("begin a 50 = temp a 2 * = end", &mut stack, &mut env).unwrap();
        assert_eq!(env.lookup("a"), Some(Value::Int(1)));
        assert_eq!(env.lookup("temp"), None);
    }
}
