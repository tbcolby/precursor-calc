"""Renode capture sequence for Calculator (not yet run).

Run from xous-dev-toolkit:
    python3 scripts/renode_capture.py --app calc --script ../precursor-calc/capture.py

Contract: run(ctl, outdir) -> number of failed steps. The app is already launched.
Keys: F1-F3 open menus, F4 closes a menu (or leaves on the main screen),
Enter evaluates / is RPN ENTER, uppercase M toggles ALG/RPN.
"""
import os


def run(ctl, outdir) -> int:
    failures = 0

    def shot(name):
        nonlocal failures
        if not ctl.try_screenshot(os.path.join(outdir, name)):
            failures += 1

    shot("01_initial.png")
    # Algebraic: precedence and unary minus
    ctl.inject_string("2+3*4")
    shot("02_alg_input.png")
    ctl.inject_line("")            # Enter: 14
    ctl.sleep(1.0)
    ctl.inject_line("-2^2")        # -4, not 4
    ctl.sleep(1.0)
    shot("03_alg_results.png")
    # F1 menu, then sqrt of 2
    ctl.timed_key("F1", after=1.0)
    shot("04_math_menu.png")
    ctl.timed_key("F4", after=1.0)  # close menu
    ctl.inject_string("1/0")
    ctl.inject_line("")
    ctl.sleep(1.0)
    shot("05_div_zero_error.png")
    ctl.inject_key(" ")            # clear input
    # RPN: 1 ENTER 2 ENTER 3 + + = 6
    ctl.inject_key("M")
    ctl.sleep(1.0)
    ctl.inject_line("1")
    ctl.inject_line("2")
    ctl.inject_string("3++")
    ctl.sleep(1.0)
    shot("06_rpn_chain.png")
    ctl.timed_key("F2", after=1.0)
    shot("07_trig_menu.png")
    ctl.timed_key("F4", after=1.0)
    ctl.timed_key("F3", after=1.0)
    shot("08_more_menu.png")
    ctl.timed_key("F4", after=1.0)
    ctl.inject_key("B")            # HEX display
    ctl.sleep(1.0)
    shot("09_hex.png")
    ctl.inject_key("B")
    ctl.inject_key("B")
    ctl.inject_key("B")            # back to DEC
    ctl.inject_key("M")            # back to ALG
    ctl.timed_key("F4", after=1.5)  # leave the app
    shot("10_left_app.png")
    return failures
