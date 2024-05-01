use eframe::egui::{RichText, ScrollArea, Ui};
use einput_device::input::{stick::StickId, triggers::TriggerId};

use crate::widgets::{
    buttons_input::ButtonsInput, stick_input::StickInput, trigger_input::TriggerInput,
};

use super::Configure;

impl Configure {
    pub fn tab_overview(&mut self, ui: &mut Ui) {
        self.show_info(ui);

        ScrollArea::vertical().show(ui, |ui| {
            let mut separate = false;

            if let Some(buttons) = self.get_input().and_then(|input| input.buttons()) {
                if !separate {
                    separate = true;
                } else {
                    ui.separator();
                }

                ui.label(RichText::new("Buttons").strong());

                ui.add(ButtonsInput::new(*buttons).available(self.device.info().input.buttons));
            }

            if self
                .get_input()
                .map(|input| {
                    input.stick(StickId::Left).is_some() || input.stick(StickId::Right).is_some()
                })
                .unwrap_or(false)
            {
                if !separate {
                    separate = true;
                } else {
                    ui.separator();
                }

                ui.label(RichText::new("Sticks").strong());

                ui.horizontal_wrapped(|ui| {
                    for id in StickId::ALL {
                        if let Some(stick) = self.get_input().and_then(|input| input.stick(id)) {
                            ui.add(StickInput::new(*stick, format!("{id:?}")));
                        }
                    }
                });
            }

            if let Some(triggers) = self.get_input().and_then(|input| input.triggers()) {
                if !separate {
                    // separate = true;
                } else {
                    ui.separator();
                }

                ui.label(RichText::new("Triggers").strong());

                ui.horizontal_wrapped(|ui| {
                    for id in TriggerId::ALL {
                        ui.add(TriggerInput::new(*triggers.get(id), format!("{id:?}")));
                    }
                });
            }
        });
    }

    fn show_info(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Name");
            ui.label(RichText::new(self.device.info().name()).strong());

            ui.add_space(20.0);

            ui.label("ID");
            ui.label(
                RichText::new(self.device.info().id().as_str())
                    .strong()
                    .monospace(),
            );

            ui.add_space(20.0);

            ui.label("Type");
            ui.label(RichText::new(format!("{:#?}", self.device.info().kind)).strong());
        });

        ui.separator();
    }
}
