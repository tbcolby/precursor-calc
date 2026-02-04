# Precursor Scientific Calculator

A full-featured **TI-85 competitor** scientific calculator for the [Precursor](https://www.crowdsupply.com/sutajio-kosagi/precursor) hardware platform, featuring both algebraic and RPN modes.

Built for the [Xous](https://github.com/betrusted-io/xous-core) microkernel OS.

---

## Features

- **Dual Mode Operation**: Standard algebraic (infix) and RPN (Reverse Polish Notation)
- **Full Operator Precedence**: Proper mathematical order of operations with parentheses support
- **Scientific Functions**: sin, cos, tan, asin, acos, atan, sinh, cosh, tanh, ln, log, log2, sqrt, cbrt, exp, abs, floor, ceil, round, factorial
- **4-Level RPN Stack**: Classic HP-style X, Y, Z, T registers with LastX recall
- **Memory Registers**: 10 memory slots (M0-M9) for storing values
- **Angle Modes**: Degrees, Radians, Gradians
- **Number Bases**: Decimal, Hexadecimal, Octal, Binary display
- **History Tape**: View and recall previous calculations
- **Persistent Settings**: Mode, angle, base, and memory saved to PDDB

---

## Screenshots

### Initial State
![Initial State](screenshots/01_initial.png)

*Clean algebraic mode interface with status indicators and function key menu*

---

### Operator Precedence

| Expression Entry | Result |
|------------------|--------|
| ![Expression](screenshots/02_expression.png) | ![Result](screenshots/03_result.png) |

**2+3×4 = 14** — Not 20! Proper mathematical order of operations.

---

### Trigonometric Functions

| Input | Result |
|-------|--------|
| ![sin(90) input](screenshots/04_sin90_input.png) | ![sin(90) result](screenshots/05_sin90_result.png) |

**sin(90) = 1** in degree mode. Full trig support with configurable angle units.

---

### Square Root

![sqrt(2)](screenshots/06_sqrt2.png)

**sqrt(2) = 1.41421356...** — Type function names directly in algebraic mode.

---

### Parentheses Support

| Expression | Result |
|------------|--------|
| ![Parentheses](screenshots/07_parens_input.png) | ![Result](screenshots/08_parens_result.png) |

**(2+3)×4 = 20** — Override precedence with parentheses.

---

### RPN Mode

![RPN Mode](screenshots/09_rpn_mode.png)

*Classic 4-level stack display (T, Z, Y, X) with LastX register*

---

### RPN Stack Operations

| Enter 2 | Push to stack | Enter 3 | Add = 5 |
|---------|---------------|---------|---------|
| ![2](screenshots/10_rpn_digit2.png) | ![Stack](screenshots/11_rpn_enter.png) | ![3](screenshots/12_rpn_digit3.png) | ![5](screenshots/13_rpn_add.png) |

**2 Enter 3 + = 5** — Classic HP-style postfix calculation.

---

### Complex RPN Calculations

| Stack with values | sqrt result |
|-------------------|-------------|
| ![Complex](screenshots/14_rpn_complex.png) | ![sqrt](screenshots/15_rpn_sqrt.png) |

*Stack operations with scientific functions*

---

### Angle Modes

| Radians Mode (RPN) | Radians Mode (ALG) |
|--------------------|---------------------|
| ![RAD RPN](screenshots/16_rad_mode.png) | ![RAD ALG](screenshots/17_alg_rad.png) |

*Toggle between DEG, RAD, and GRAD with the 'A' key*

---

### Memory Operations

![Memory Store](screenshots/18_memory_store.png)

*10 memory registers (M0-M9) — Store with S+digit, Recall with K+digit*

---

### Number Base Conversion

![Hex Mode](screenshots/19_hex_mode.png)

*Cycle through DEC, HEX, OCT, BIN display modes with 'B' key*

---

## Keyboard Controls

### Basic Keys
| Key | Function |
|-----|----------|
| `0-9` | Digit entry |
| `.` | Decimal point |
| `+` `-` `*` `/` | Basic operators |
| `^` | Power (x^y) |
| `%` | Modulo |
| `(` `)` | Parentheses (algebraic mode) |
| `Enter` / `=` | Execute (algebraic) / Push (RPN) |
| `Backspace` | Delete character |
| `Space` | Clear entry (CLx) |
| `C` | Clear all (AC) |

### Mode Controls
| Key | Function |
|-----|----------|
| `M` | Toggle ALG/RPN mode |
| `A` | Cycle angle mode (DEG→RAD→GRAD) |
| `B` | Cycle number base (DEC→HEX→OCT→BIN) |

### RPN-Specific Keys
| Key | Function |
|-----|----------|
| `x` / `X` | Swap X↔Y |
| `r` / `R` | Roll stack down |
| `l` / `L` | Recall LastX |

### Memory Operations
| Key | Function |
|-----|----------|
| `S` + `0-9` | Store to memory M0-M9 |
| `K` + `0-9` | Recall from memory M0-M9 |

### Shift Layer (Shift + Key)
| Key | Function |
|-----|----------|
| `Shift+1` | sin |
| `Shift+2` | cos |
| `Shift+3` | tan |
| `Shift+4` | ln |
| `Shift+5` | log |
| `Shift+6` | sqrt |
| `Shift+7` | x² |
| `Shift+8` | 1/x |
| `Shift+.` | π |
| `Shift+e` | e |
| `Shift+-` | Change sign (+/-) |

---

## Function Menus

Press F1-F4 for function menus, then press 0-9 to select:

- **F1: MATH** — abs, floor, ceil, round, mod, !, exp, 10^x, x³, ³√x
- **F2: TRIG** — sin, cos, tan, asin, acos, atan, sinh, cosh, tanh
- **F3: MODE** — Toggle mode, angle, base
- **F4: MEM** — Memory operations

---

## Building

```bash
# Clone xous-core if you haven't already
git clone https://github.com/betrusted-io/xous-core.git
cd xous-core

# Copy this app to apps/calc
# Add to workspace Cargo.toml and apps/manifest.json

# For Renode emulator
cargo xtask renode-image calc

# For real Precursor hardware
cargo xtask app-image calc
```

---

## Technical Notes

- All calculations use 64-bit floating point (f64)
- Factorial uses gamma function for non-integers
- Settings are persisted to PDDB dictionary `calc.settings`
- Display supports scientific notation for very large/small numbers
- Non-decimal bases display integers only
- Expression parser uses shunting-yard algorithm for proper precedence

---

## Author

**Made by Tyler Colby — Colby's Data Movers, LLC**

- Email: tyler@colbysdatamovers.com
- Issues: [GitHub Issues](https://github.com/tylercolby/precursor-calc/issues)

---

## License

Apache License, Version 2.0

```
Copyright 2025 Tyler Colby, Colby's Data Movers, LLC

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
