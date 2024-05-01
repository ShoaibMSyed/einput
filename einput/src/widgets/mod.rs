use eframe::egui::{Color32, Context, Ui};

pub mod buttons_input;
pub mod device_preview;
pub mod device_selector;
pub mod stick_input;
pub mod trigger_input;

pub trait GetExtraVisuals {
    fn evisuals(&self) -> &ExtraVisuals;
}

pub struct ExtraVisuals {
    pub panel_color: Color32,
}

impl ExtraVisuals {
    const LIGHT: &'static ExtraVisuals = &ExtraVisuals {
        panel_color: Color32::from_rgb(0xF2, 0xF2, 0xF2),
    };

    const DARK: &'static ExtraVisuals = &ExtraVisuals {
        panel_color: Color32::from_rgb(0x15, 0x15, 0x15),
    };
}

impl GetExtraVisuals for Ui {
    fn evisuals(&self) -> &ExtraVisuals {
        if self.visuals().dark_mode {
            ExtraVisuals::DARK
        } else {
            ExtraVisuals::LIGHT
        }
    }
}

impl GetExtraVisuals for Context {
    fn evisuals(&self) -> &ExtraVisuals {
        if self.style().visuals.dark_mode {
            ExtraVisuals::DARK
        } else {
            ExtraVisuals::LIGHT
        }
    }
}
