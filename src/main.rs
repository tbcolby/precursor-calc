//! Precursor Scientific Calculator
//!
//! A TI-85 competitor with algebraic and RPN modes, scientific functions,
//! memory registers, and persistent settings.

#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

extern crate alloc;

mod algebraic;
mod app;
mod display;
mod functions;
mod keymap;
mod memory;
mod rpn;
mod storage;
mod ui;

use app::CalcApp;
use num_traits::FromPrimitive;

// Server name for xous names registration (underscored)
const SERVER_NAME: &str = "_Calc Scientific_";
// App name for GAM registration (must match manifest context_name)
const APP_NAME: &str = "Calculator";

#[derive(Debug, num_derive::FromPrimitive, num_derive::ToPrimitive)]
enum CalcOp {
    Redraw = 0,
    Rawkeys,
    FocusChange,
    Quit,
}

fn main() -> ! {
    // Initialize logging
    log_server::init_wait().unwrap();
    log::set_max_level(log::LevelFilter::Info);
    log::info!("Calculator starting, PID {}", xous::process::id());

    // Connect to name server and register
    let xns = xous_names::XousNames::new().unwrap();
    let sid = xns
        .register_name(SERVER_NAME, None)
        .expect("can't register server");

    // Connect to GAM for graphics
    let gam = gam::Gam::new(&xns).expect("can't connect to GAM");

    // Register UX with GAM
    let token = gam
        .register_ux(gam::UxRegistration {
            app_name: String::from(APP_NAME),
            ux_type: gam::UxType::Chat,
            predictor: None,
            listener: sid.to_array(),
            redraw_id: CalcOp::Redraw.to_u32().unwrap(),
            gotinput_id: None,
            audioframe_id: None,
            rawkeys_id: Some(CalcOp::Rawkeys.to_u32().unwrap()),
            focuschange_id: Some(CalcOp::FocusChange.to_u32().unwrap()),
        })
        .expect("couldn't register UX")
        .unwrap();

    // Get drawing canvas
    let content = gam.request_content_canvas(token).expect("couldn't get canvas");
    let screensize = gam.get_canvas_bounds(content).expect("couldn't get dimensions");
    log::info!("Canvas size: {:?}", screensize);

    // Initialize calculator app
    let mut calc = CalcApp::new();
    let mut allow_redraw = true;

    // Initial draw
    calc.draw(&gam, content);

    // Main event loop
    loop {
        let msg = xous::receive_message(sid).unwrap();
        match FromPrimitive::from_usize(msg.body.id()) {
            Some(CalcOp::Redraw) => {
                if allow_redraw {
                    calc.draw(&gam, content);
                }
            }
            Some(CalcOp::Rawkeys) => xous::msg_scalar_unpack!(msg, k1, k2, k3, k4, {
                let keys = [
                    core::char::from_u32(k1 as u32).unwrap_or('\u{0000}'),
                    core::char::from_u32(k2 as u32).unwrap_or('\u{0000}'),
                    core::char::from_u32(k3 as u32).unwrap_or('\u{0000}'),
                    core::char::from_u32(k4 as u32).unwrap_or('\u{0000}'),
                ];

                let mut needs_redraw = false;
                let mut should_quit = false;

                for &key in keys.iter() {
                    if key != '\u{0000}' {
                        log::debug!("Key: {:?} (0x{:04X})", key, key as u32);

                        // Handle the key - let keymap decide actions including quit
                        if !calc.handle_key(key) {
                            // handle_key returns false for Quit action
                            should_quit = true;
                            break;
                        }
                        needs_redraw = true;
                    }
                }

                if should_quit {
                    break;
                }

                if needs_redraw && allow_redraw {
                    calc.draw(&gam, content);
                }
            }),
            Some(CalcOp::FocusChange) => xous::msg_scalar_unpack!(msg, state_code, _, _, _, {
                match gam::FocusState::convert_focus_change(state_code) {
                    gam::FocusState::Background => {
                        allow_redraw = false;
                        // Save state when going to background
                        calc.save_state();
                    }
                    gam::FocusState::Foreground => {
                        allow_redraw = true;
                        calc.draw(&gam, content);
                    }
                }
            }),
            Some(CalcOp::Quit) => break,
            _ => log::warn!("unknown opcode: {:?}", msg.body.id()),
        }
    }

    // Save state before exit
    calc.save_state();

    // Cleanup
    xns.unregister_server(sid).unwrap();
    xous::destroy_server(sid).unwrap();
    xous::terminate_process(0)
}

// Use num_traits for to_u32
use num_traits::ToPrimitive;
