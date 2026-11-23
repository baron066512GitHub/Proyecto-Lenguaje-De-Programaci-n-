//! Punto de entrada de la aplicación interactiva de consola (Mini-REPL en RPN).
//!
//! Demuestra el ciclo principal de lectura-evaluación-impresión (REPL) con
//! estructuras de control expresivas (`'repl: loop`, `match`), manejo de I/O
//! seguro con `Result` y préstamo explícito de `EvalStack` y `Environment`.

use std::io::{self, BufRead, Write};

use rpn_repl::environment::{describe_scope_state, Environment, SymbolTable};
use rpn_repl::evaluator::{evaluate_line, EvalOutcome};
use rpn_repl::stack::EvalStack;

const BANNER: &str = "\
============================================================
  Mini-REPL Postfijo (RPN) en Rust  |  CAO604 - FACYT UC
  Escriba 'help' para ver comandos o 'exit' para finalizar.
============================================================";

/// Construye el indicador de entrada (*prompt*) reflejando el ámbito léxico activo.
fn format_prompt(env: &dyn SymbolTable) -> String {
    match env.depth() {
        0 => String::from("rpn[global]> "),
        level => format!("rpn[local:{}]> ", level),
    }
}

/// Presenta al usuario el resultado de evaluar una línea sin abortar ante errores.
fn render_outcome(outcome: EvalOutcome, env: &dyn SymbolTable) -> bool {
    match outcome {
        EvalOutcome::Empty => false,
        EvalOutcome::Value(val) => {
            println!("=> {} : {}", val, val.type_name());
            false
        }
        EvalOutcome::MultipleValues(values) => {
            let rendered: Vec<String> = values
                .iter()
                .map(|v| format!("{} : {}", v, v.type_name()))
                .collect();
            println!("=> [{}]", rendered.join(", "));
            false
        }
        EvalOutcome::Messages(msgs) => {
            for msg in msgs {
                println!("{}", msg);
            }
            false
        }
        EvalOutcome::Exit => {
            println!("Sesión finalizada. {}", describe_scope_state(env));
            true
        }
    }
}

fn main() {
    println!("{}", BANNER);

    // El estado del evaluador vive en el marco de `main` y se presta explícitamente
    // (`&mut stack`, `&mut env`) a las subrutinas de evaluación en cada iteración.
    let mut stack = EvalStack::new();
    let mut env = Environment::new();

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut stdout = io::stdout();
    let mut line_buffer = String::new();

    'repl: loop {
        print!("{}", format_prompt(&env));
        if let Err(err) = stdout.flush() {
            eprintln!("Error de salida estándar: {}", err);
            break 'repl;
        }

        line_buffer.clear();
        match reader.read_line(&mut line_buffer) {
            Ok(0) => {
                // Fin de flujo (EOF / Ctrl+D / Ctrl+Z)
                println!("\nFin de entrada detectado. Saliendo...");
                break 'repl;
            }
            Ok(_) => match evaluate_line(&line_buffer, &mut stack, &mut env) {
                Ok(outcome) => {
                    if render_outcome(outcome, &env) {
                        break 'repl;
                    }
                }
                Err(eval_err) => {
                    // Manejo controlado: informa el error y continúa el REPL sin abortar.
                    eprintln!("[ERROR] {}", eval_err);
                }
            },
            Err(io_err) => {
                eprintln!("[ERROR I/O] No se pudo leer la entrada: {}", io_err);
                break 'repl;
            }
        }
    }
}
