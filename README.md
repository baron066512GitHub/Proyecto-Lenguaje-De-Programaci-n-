# Mini-REPL Evaluador Postfijo (RPN) en Rust

**Universidad de Carabobo — Facultad Experimental de Ciencias y Tecnología (FACYT)**  
**U.A. de Algoritmos, Programación y Lenguajes — CAO604: Lenguajes de Programación**  
**Asignación #1 — Semestre 1-2026**

Aplicación interactiva de consola (REPL) desarrollada en **Rust (edición 2021, sin dependencias externas)** que evalúa expresiones matemáticas y lógicas en **Notación Polaca Inversa (Postfija / RPN)** estricta, diseñada para ejemplificar de forma práctica el diseño e implementación de Rust:

1. **Nombres, vinculación (*binding*), estructuras de control y reglas de alcance (*scoping*).**
2. **Sistema de tipos estático, ADTs (`enum`), inferencia, equivalencia nominal y manejo idiomático de errores (`Result`/`Option`).**
3. **Subrutinas, paso explícito de parámetros (*move*, `Copy`, `&T`, `&mut T`) y manejo determinista de memoria (Ownership / RAII).**
4. **Abstracción de datos, encapsulamiento modular, composición y polimorfismo mediante *traits* (`SymbolTable`, `Evaluable`, `Display`, `Error`).**

---

## Estructura del Proyecto

```text
rpn-repl/
├── Cargo.toml            # Metadatos del paquete (cero dependencias externas)
├── README.md             # Guía de compilación, ejecución y sesión de prueba
└── src/
    ├── main.rs           # Ciclo interactivo REPL, manejo de E/S y prompt por ámbito
    ├── lib.rs            # Exportación de módulos y suite de pruebas unitarias/integración
    ├── types.rs          # ADTs: Value (Int/Float/Bool), Operator, Command, Token, StackEntry
    ├── error.rs          # Enum EvalError e implementaciones de Display y std::error::Error
    ├── lexer.rs          # Tokenizador separado por espacios con validación de literales e identificadores
    ├── stack.rs          # Estructura encapsulada EvalStack con protección contra underflow
    ├── environment.rs    # Trait SymbolTable y struct Environment (pila de ámbitos y shadowing)
    └── evaluator.rs      # Trait Evaluable y motor RPN con chequeo estricto de tipos
```

---

## Requisitos Previos

- **Rust y Cargo** (versión 1.70 o superior; probado con `rustc 1.100.0` / `cargo 1.100.0`).
- Instalación oficial: [https://rustup.rs](https://rustup.rs)

---

## Cómo Compilar y Ejecutar

Desde la carpeta `rpn-repl/`:

### 1. Compilar el proyecto
```bash
cargo build
```
Para compilación optimizada (*release*):
```bash
cargo build --release
```

### 2. Ejecutar el Mini-REPL interactivo
```bash
cargo run
```

### 3. Ejecutar la suite de pruebas automatizadas
```bash
cargo test
```

### 4. Verificación estática con Clippy (opcional)
```bash
cargo clippy --all-targets -- -D warnings
```

---

## Sintaxis RPN y Comandos Soportados

Todos los tokens se separan por espacios en blanco:

| Categoría | Elementos | Ejemplo RPN | Equivalente Infijo / Efecto |
| :--- | :--- | :--- | :--- |
| **Literales** | `Int` (`i64`), `Float` (`f64`), `Bool` (`true`/`false`) | `42`, `-3.5`, `true` | Valores tipados en pila |
| **Aritméticos** | `+`, `-`, `*`, `/` | `x 5 + 2 *` | `(x + 5) * 2` |
| **Relacionales** | `==`, `!=`, `<`, `>`, `<=`, `>=` | `10 5 >` | `10 > 5` (`true`) |
| **Lógicos** | `and`, `or`, `not` | `true false and not` | `!(true && false)` (`true`) |
| **Asignación** | `=` (`<ident> <expr> =`) | `x 5 2 + =` | `x = 5 + 2` |
| **Ámbito local** | `begin`, `end` | `begin` ... `end` | Abre/cierra entorno local con *shadowing* |
| **Inspección** | `vars` | `vars` | Lista variables por nivel de ámbito |
| **Control REPL** | `help`, `exit` / `quit` | `help` | Ayuda rápida / finalizar sesión |

### Reglas de Tipos del Evaluador
- `Int op Int -> Int` (con verificación de desbordamiento y división por cero).
- `Int op Float`, `Float op Int`, `Float op Float -> Float` (promoción numérica explícita).
- Operaciones aritméticas u orden (`<`, `>`) con `Bool` generan `EvalError::TypeMismatch`.
- Igualdad (`==`, `!=`) requiere operandos del mismo dominio (ambos numéricos o ambos `Bool`).
- Operadores lógicos (`and`, `or`, `not`) aceptan únicamente `Bool`.

---

## Sesión de Prueba de Ejemplo

A continuación se muestra una sesión real ejecutada con `cargo run --quiet`, cubriendo evaluación aritmética y lógica, asignación de variables, errores controlados y *scoping/shadowing* con `begin`/`end`:

```text
============================================================
  Mini-REPL Postfijo (RPN) en Rust  |  CAO604 - FACYT UC
  Escriba 'help' para ver comandos o 'exit' para finalizar.
============================================================
rpn[global]> x 10 =
x = 10 : Int
rpn[global]> x 5 + 2 *
=> 30 : Int
rpn[global]> y 3.5 =
y = 3.5 : Float
rpn[global]> x y +
=> 13.5 : Float
rpn[global]> x 10 ==
=> true : Bool
rpn[global]> 5 true +
[ERROR] Error de tipos: '+' esperaba operandos numéricos (Int o Float), pero recibió (Int, Bool)
rpn[global]> 10 0 /
[ERROR] Error aritmético: división por cero
rpn[global]> 5 +
[ERROR] Error de pila (stack underflow): '+' requiere 2 operando(s), pero solo hay 1 en la pila
rpn[global]> 12abc 1 +
[ERROR] Error léxico: token inválido '12abc'
rpn[global]> begin
Bloque local abierto (nivel de alcance: 1)
rpn[local:1]> x 99 =
x = 99 : Int
rpn[local:1]> z 5 =
z = 5 : Int
rpn[local:1]> x z +
=> 104 : Int
rpn[local:1]> vars
[Ámbito 0 - global]
  x = 10 : Int [sombreada]
  y = 3.5 : Float
[Ámbito 1 - local]
  x = 99 : Int
  z = 5 : Int
rpn[local:1]> end
Bloque local cerrado (retorno a ámbito global)
rpn[global]> x
=> 10 : Int
rpn[global]> z
[ERROR] Error de nombre: la variable 'z' no está definida en el ámbito actual
rpn[global]> vars
[Ámbito 0 - global]
  x = 10 : Int
  y = 3.5 : Float
rpn[global]> end
[ERROR] Error de alcance: no hay ningún bloque local abierto que cerrar con 'end'
rpn[global]> exit
Sesión finalizada. Entorno [global]:
[Ámbito 0 - global]
  x = 10 : Int
  y = 3.5 : Float
```
