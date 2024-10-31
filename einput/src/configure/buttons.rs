use eframe::egui::{ComboBox, Frame, ScrollArea, Stroke, Ui};
use einput_device::input::buttons::Button;

use super::Configure;

impl Configure {
    pub fn tab_buttons(&mut self, ui: &mut Ui) {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for button in Button::ALL {
                    let stroke = if self
                        .get_raw_input()
                        .and_then(|input| input.get(button))
                        .unwrap_or(false)
                    {
                        ui.visuals().widgets.active.fg_stroke
                    } else {
                        Stroke::NONE
                    };

                    Frame::none()
                        .inner_margin(2.0)
                        .stroke(stroke)
                        .show(ui, |ui| {
                            let mut selected = None;

                            ComboBox::from_label(format!("{button:?}"))
                                .selected_text(format!(
                                    "{:?}",
                                    self.config.input.buttons[button as usize]
                                ))
                                .show_ui(ui, |ui| {
                                    for button in Button::ALL {
                                        ui.selectable_value(
                                            &mut selected,
                                            Some(button),
                                            format!("{button:?}"),
                                        );
                                    }
                                });

                            if let Some(new) = selected {
                                self.config.input.buttons[button as usize] = new;
                                self.update_config();
                            }
                        });
                }
            });
    }
}
