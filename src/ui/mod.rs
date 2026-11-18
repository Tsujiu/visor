//! Módulo de interface gráfica, tema e apresentação visual.

#![allow(unused_imports)]

pub mod banner;
pub mod canvas;
pub mod color_picker;
pub mod context_menu;
pub mod dialogs;
pub mod filmstrip;
pub mod info_panel;
pub mod settings;
pub mod status;
pub mod support_dialog;
pub mod theme;
pub mod toolbar;
pub mod update_dialog;
pub mod whats_new;

pub use banner::{render_association_banner, render_update_banner};
pub use canvas::render_canvas;
pub use color_picker::render_color_picker_window;
pub use context_menu::render_context_menu;
pub use dialogs::{render_batch_window, render_formats_window};
pub use filmstrip::render_filmstrip;
pub use info_panel::render_info_panel;
pub use settings::render_settings_window;
pub use status::render_status;
pub use support_dialog::render_support_dialog;
pub use theme::{
    ThemeDimensions, ThemePalette, ThemeRounding, ThemeSpacing, configure_visor_theme,
};
pub use toolbar::render_toolbar;
pub use update_dialog::render_update_dialog;
pub use whats_new::render_whats_new_window;
