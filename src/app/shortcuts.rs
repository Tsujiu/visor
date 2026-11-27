//! Mapeamento e despacho de teclas de atalho e botões do mouse.

use eframe::egui::{self, Key, Modifiers, PointerButton, ViewportCommand};

use super::state::VisorApp;

pub fn handle_keys(app: &mut VisorApp, ui: &mut egui::Ui) {
    let open = ui.input_mut(|input| input.consume_key(Modifiers::COMMAND, Key::O));
    if open {
        app.want_open = true;
    }
    let export = ui.input_mut(|input| input.consume_key(Modifiers::COMMAND, Key::S));
    if export {
        app.want_export = true;
    }
    let settings = ui.input_mut(|input| input.consume_key(Modifiers::COMMAND, Key::Comma));
    if settings {
        app.show_settings = !app.show_settings;
    }

    if !ui.ctx().egui_wants_keyboard_input() {
        let copy = ui.input_mut(|input| input.consume_key(Modifiers::COMMAND, Key::C));
        if copy {
            app.copy_current_image(ui.ctx());
        }

        let delete = ui.input_mut(|input| {
            let plain = !input.modifiers.command && !input.modifiers.alt && !input.modifiers.shift;
            if plain && input.key_pressed(Key::Delete) {
                input.consume_key(Modifiers::NONE, Key::Delete);
                true
            } else {
                false
            }
        });
        if delete {
            app.delete_current_image(ui.ctx());
        }
    }

    let (
        plain_left,
        plain_right,
        home,
        end,
        fit,
        actual,
        rotate,
        shift,
        fullscreen,
        info,
        space,
        pixel_toggle,
        bg_toggle,
        filmstrip_toggle,
    ) = ui.input(|input| {
        let plain = !input.modifiers.command && !input.modifiers.alt && !input.modifiers.shift;
        (
            plain && (input.key_pressed(Key::ArrowLeft) || input.key_pressed(Key::PageUp)),
            plain && (input.key_pressed(Key::ArrowRight) || input.key_pressed(Key::PageDown)),
            plain && input.key_pressed(Key::Home),
            plain && input.key_pressed(Key::End),
            plain && input.key_pressed(Key::F),
            plain && (input.key_pressed(Key::Num1) || input.key_pressed(Key::Num0)),
            !input.modifiers.command && !input.modifiers.alt && input.key_pressed(Key::R),
            input.modifiers.shift,
            plain && input.key_pressed(Key::F11),
            plain && input.key_pressed(Key::I),
            plain && input.key_pressed(Key::Space),
            plain && input.key_pressed(Key::P),
            plain && input.key_pressed(Key::B),
            plain && input.key_pressed(Key::T),
        )
    });

    if plain_left {
        app.step(-1);
    }
    if plain_right {
        app.step(1);
    }
    if home {
        app.go_to(0);
    }
    if end {
        app.go_to(app.files.len().saturating_sub(1));
    }
    if fit {
        app.view.set_fit();
    }
    if actual {
        let ppp = ui.pixels_per_point();
        app.view.set_actual(ppp);
    }
    if rotate {
        app.rotate(ui.ctx(), !shift);
    }
    if info {
        app.show_info = !app.show_info;
    }
    if filmstrip_toggle {
        app.show_filmstrip = !app.show_filmstrip;
    }
    if pixel_toggle {
        app.interpolation = match app.interpolation {
            crate::app::state::InterpolationMode::Nearest => {
                crate::app::state::InterpolationMode::Auto
            }
            _ => crate::app::state::InterpolationMode::Nearest,
        };
    }
    if bg_toggle {
        app.background = app.background.cycle();
    }
    if space
        && let Some(picture) = app.picture.as_mut()
            && picture.frames.len() > 1 {
                picture.playing = !picture.playing;
                picture.shown_at = ui.input(|input| input.time);
            }
    if fullscreen {
        let on = ui
            .ctx()
            .input(|input| input.viewport().fullscreen.unwrap_or(false));
        ui.ctx().send_viewport_cmd(ViewportCommand::Fullscreen(!on));
    }

    let escape_fullscreen = ui.input(|input| {
        input.key_pressed(Key::Escape)
            && input.viewport().fullscreen.unwrap_or(false)
            && !input.modifiers.command
    });
    if escape_fullscreen {
        ui.ctx()
            .send_viewport_cmd(ViewportCommand::Fullscreen(false));
    }

    let (back, forward) = ui.input(|input| {
        (
            input.pointer.button_pressed(PointerButton::Extra1),
            input.pointer.button_pressed(PointerButton::Extra2),
        )
    });
    if back {
        app.step(-1);
    }
    if forward {
        app.step(1);
    }
}
