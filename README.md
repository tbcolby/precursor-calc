# Precursor Scientific Calculator

A scientific calculator for [Precursor](https://precursor.dev) on the Xous OS, with an algebraic (infix) mode and an HP-style four-level RPN mode. All arithmetic is 64-bit floating point.

Status: builds at the toolkit's xous-core anchor, clippy-clean, host tests pass (`cd core && cargo test`). **Not yet verified on Renode or hardware**: `capture.py` is written but the Renode capture hasn't been run. Screenshots in `screenshots/` are from version 0.1 and show the old layout.

## Features

- **Algebraic mode**: type an expression, Enter evaluates. Usual precedence; `^` is right-associative; unary minus binds looser than `^` (`-2^2 = -4`) and tighter than `*`. A function applies to the operand right after it (`sin 30+1 = sin(30)+1`, `-sqrt(4)+1 = -1`). Postfix `!`. Implicit multiplication (`2pi`, `2(3)`, `2sqrt(9)`). An expression that starts with `+ * / ^ % !` continues the previous answer (`*2` means `ans*2`); a leading `-` is negation. Scientific literals (`1.5e-3`).
- **RPN mode**: X, Y, Z, T registers and LASTx with real HP stack lift: ENTER copies X to Y and disables lift, CLx disables lift, operations enable it. `1 ENTER 2 ENTER 3 + +` is 6. EEX and CHS during entry. A failed operation leaves the stack as it was.
- **Functions** (F-key menus): sqrt, x², x³, 1/x, ln, log, log2, eˣ, 10ˣ, n! (gamma for non-integers), abs, floor, ceil, round, cbrt, sin/cos/tan and inverses, sinh/cosh/tanh, asinh, π, e.
- **Errors, never crashes**: division by zero, overflow (any non-finite result), domain errors and syntax errors are shown on screen. Exact trig at quarter turns in DEG/GRAD (`sin(180) = 0`); `tan` at a pole is an error.
- **Display**: up to 12 significant digits (no `12345678.9000000004`), scientific notation from 1e12 and below 1e-4. HEX/OCT/BIN show the integer part: a `~` prefix marks a truncated fraction, values beyond 64-bit show "out of range".
- **Memory** M0–M9, **angle modes** DEG/RAD/GRAD, **display bases** DEC/HEX/OCT/BIN.
- **History tape** of algebraic results; ↑ ↓ recall earlier expressions into the input line.

## Keys

`∴` opens the system menu and never reaches the app. There is no Shift layer: upper-case letters are commands, lower-case letters are typed into expressions (algebraic) or are stack commands (RPN).

| Where | Key | Action |
|---|---|---|
| Both modes | `0-9` `.` | Number entry |
| | `+ - * / ^ %` | Operators (`%` is remainder with the sign of the dividend, not percent) |
| | Enter, `=` | Evaluate (ALG) / ENTER (RPN) |
| | Backspace | Delete a character (RPN outside entry: CLx) |
| | Space | Clear the input line (ALG) / CLx (RPN) |
| | `C` | Clear all (ALG: input and ans; RPN: stack and LASTx) |
| | `M` / `A` / `B` | Toggle ALG/RPN / cycle angle mode / cycle display base |
| | `S` then `0-9` | Store ans (ALG) or X (RPN) in a register |
| | `K` then `0-9` | Recall a register (the prompt lists their values) |
| | F1 / F2 / F3 | Math / Trig / More menu |
| | F4 | Leave the app (it stays running; state is saved if it changed) |
| Algebraic | `a-z` `( ) !` | Type function names, `ans`, `pi`, `e` |
| | ↑ ↓ | Previous / next expression from history |
| RPN | `n` | CHS (change sign; the exponent's sign during EEX) |
| | `e` | EEX (exponent entry) |
| | `x` | Swap X and Y |
| | `r`, ↓ / `u`, ↑ | Roll down / roll up |
| | `l` | LASTx |
| | `!` | Factorial |
| Menus and prompts | listed key | Choose the item |
| | same F-key, F4, Backspace | Close |
| | another F1–F3 | Switch menu |

Menus: **F1 Math** `1` sqrt `2` x² `3` 1/x `4` ln `5` log `6` eˣ `7` 10ˣ `8` n! `9` π `0` e. **F2 Trig** `1-3` sin cos tan `4-6` asin acos atan `7-9` sinh cosh tanh `0` asinh. **F3 More** `1` abs `2` floor `3` ceil `4` round `5` cbrt `6` x³ `7` log2 `8` +/- `9` STO `0` RCL `m` ALG/RPN `a` angle `b` base `c` clear memory `w` overwrite an unreadable save.

In algebraic mode a menu function inserts parseable text (`sqrt(`, `^2`, `!`, `1/(`), and +/- wraps the whole input in `-( )`.

## Storage

| PDDB dictionary | Key | Contents |
|---|---|---|
| `calc.settings` | `state` | JSON: `version`, mode, angle mode, base, M0–M9, ans, RPN stack and LASTx, last 20 history entries |

- Written through `precursor-appkit`'s `Store` (delete then rewrite in the key's basis, then sync), so a shorter record never leaves a stale tail. Version 0.1 wrote over the old value in place, which corrupted the record whenever it got shorter.
- Saved only when the state changed, on leaving the app or losing focus.
- Reloaded when a PDDB basis is locked or unlocked.
- Version 0.1 records still load. A record that can't be parsed, or that a newer version wrote, is not overwritten; the status line says so, and F3 `w` lets you overwrite it deliberately.
- Non-finite values can't be stored (all results are checked), so the record always round-trips.

## Limitations

- HEX/OCT/BIN are display-only; entry is always decimal.
- No complex numbers, units, equation solver, graphing or programmability.
- The history tape records algebraic evaluations only, not RPN operations.
- Long algebraic input scrolls to show its end; there is no cursor movement inside the expression.
- Screen layout hasn't been checked on a device yet. Renode capture not yet run.

## Building

Builds inside a [xous-core](https://github.com/betrusted-io/xous-core) tree. With [xous-dev-toolkit](https://github.com/tbcolby/xous-dev-toolkit):

```bash
bin/integrate --xous-root ~/xous-core --grade ~/Dev/precursor/precursor-calc   # build, clippy, tests, lint
```

Host tests alone: `cd core && cargo test`.

## Layout

```
core/src/functions.rs   functions, operators, errors (never NaN/inf)
core/src/parser.rs      recursive-descent infix parser
core/src/rpn.rs         four-level stack with HP stack lift
core/src/format.rs      display formatting
core/src/menu.rs        F1-F3 menus
core/src/calculator.rs  key-driven controller and saved state
core/src/tests.rs       key-sequence and regression tests
src/main.rs             GAM registration and message loop
src/app.rs              persistence glue
src/ui.rs               drawing
capture.py              Renode capture sequence (not yet run)
```

## Author

Tyler Colby, [Colby's Data Movers, LLC](https://colbysdatamovers.com). Licensed under the Apache License, Version 2.0 (see [LICENSE](LICENSE)).
