//! Scientific calculator for Precursor: algebraic and HP-style RPN modes.
//!
//! All calculator logic lives in `calc-core` (host-tested). This crate
//! registers with GAM, routes messages, persists, and draws.

mod app;
mod ui;

use num_traits::{FromPrimitive, ToPrimitive};
use precursor_appkit::{app as kit, keys, store};

/// xous-names server name. Any unique string; NOT the GAM app name.
const SERVER_NAME: &str = "_Calc Scientific_";

#[derive(Debug, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub(crate) enum AppOp {
    Redraw = 0,
    Rawkeys,
    FocusChange,
    BasisChanged,
    Quit,
}

fn main() -> ! {
    log_server::init_wait().unwrap();
    log::set_max_level(log::LevelFilter::Info);
    log::info!("calc PID is {}", xous::process::id());

    let xns = xous_names::XousNames::new().unwrap();
    let sid = xns.register_name(SERVER_NAME, None).expect("can't register server");

    // GAM app name comes from apps/manifest.json key "calc".
    let reg = kit::register(
        &xns,
        sid,
        gam::APP_NAME_CALC,
        gam::UxType::Framebuffer,
        kit::Opcodes {
            redraw: AppOp::Redraw.to_u32().unwrap(),
            rawkeys: Some(AppOp::Rawkeys.to_u32().unwrap()),
            focus_change: Some(AppOp::FocusChange.to_u32().unwrap()),
            got_input: None,
        },
    );

    let store = store::Store::open(app::DICT);
    store::watch_bases(xous::connect(sid).unwrap(), AppOp::BasisChanged.to_usize().unwrap());

    let mut app = app::App::new(reg.screen, store);
    let mut foreground = true;

    loop {
        let msg = xous::receive_message(sid).unwrap();
        match FromPrimitive::from_usize(msg.body.id()) {
            Some(AppOp::Redraw) => {
                if foreground {
                    app.draw();
                }
            }
            Some(AppOp::Rawkeys) => xous::msg_scalar_unpack!(msg, k1, k2, k3, k4, {
                for key in keys::decode_rawkeys([k1, k2, k3, k4]) {
                    if app.on_key(key) == calc_core::Action::Leave {
                        app.save();
                        kit::leave(app.screen());
                    }
                }
                if foreground {
                    app.draw();
                }
            }),
            Some(AppOp::FocusChange) => xous::msg_scalar_unpack!(msg, code, _, _, _, {
                match kit::focus_state(code) {
                    gam::FocusState::Foreground => {
                        foreground = true;
                        app.on_foreground();
                    }
                    gam::FocusState::Background => {
                        foreground = false;
                        app.save();
                    }
                }
            }),
            Some(AppOp::BasisChanged) => {
                app.reload();
                if foreground {
                    app.draw();
                }
            }
            Some(AppOp::Quit) => break,
            None => log::warn!("unknown opcode {:?}", msg.body.id()),
        }
    }
    app.save();
    kit::shutdown(&xns, sid)
}
