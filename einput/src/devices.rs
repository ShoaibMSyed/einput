use eframe::egui::{
    self, Align, Button, Frame, Id, Label, Layout, Margin, RichText, ScrollArea, Ui,
};

use crate::{
    widgets::{device_preview::DevicePreview, GetExtraVisuals},
    App, ConfigureState,
};

impl App {
    pub fn central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(RichText::new("Controllers").strong());

            ScrollArea::vertical().show(ui, |ui| {
                let map = self.reader.current();

                for id in &self.tracking_order {
                    let dev = self.tracking.get(id).unwrap();
                    if !dev.owned() {
                        continue;
                    }

                    let Some(input) = map.get(id) else { continue };

                    Frame::none()
                        .inner_margin(Margin::same(10.0))
                        .rounding(5.0)
                        .fill(ui.evisuals().panel_color)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.dnd_drag_source(
                                    Id::new(dev.info().id()),
                                    dev.clone(),
                                    |ui: &mut Ui| {
                                        Frame::none().show(ui, |ui| {
                                            ui.add(
                                                Label::new(
                                                    RichText::new(
                                                        String::new() + "🎮  " + dev.info().name(),
                                                    )
                                                    .strong(),
                                                )
                                                .selectable(false),
                                            );
                                            ui.add_space(5.0);
                                            ui.add(
                                                Label::new(
                                                    RichText::new(dev.info().id().as_str())
                                                        .weak()
                                                        .monospace(),
                                                )
                                                .selectable(false),
                                            );
                                        });
                                    },
                                );

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.add(Button::new("Configure").rounding(5.0)).clicked() {
                                        if !self
                                            .configuring
                                            .iter()
                                            .any(|state| &state.id == dev.info().id())
                                        {
                                            self.configuring.push(ConfigureState::new(
                                                self.einput.clone(),
                                                dev.clone(),
                                                self.configs.clone(),
                                            ));
                                        }
                                    }

                                    ui.add(DevicePreview::new(&dev.info(), input));
                                });
                            });
                        });
                }
            });
        });
    }
}
