use std::collections::HashMap;

use eframe::egui::{self, Context, RichText, ScrollArea};
use einput_core::output::Output;

use crate::{
    widgets::device_selector::{DeviceSelector, PickState},
    App,
};

#[allow(unused_variables, unused_mut)]
pub fn all() -> HashMap<String, Box<dyn Output>> {
    let mut outputs = HashMap::new();

    #[cfg(windows)]
    {
        outputs.insert("vigem".to_owned(), Box::new(einput_output_vigem::XboxOutput::new()) as _);
    }

    outputs
}

impl App {
    pub fn bottom_panel(&mut self, ctx: &Context) {
        egui::TopBottomPanel::bottom("output_panel").show(ctx, |ui| {
            ui.add_space(7.0);
            ui.label(RichText::new("Output").strong());
            ui.add_space(5.0);

            ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    for output in self.outputs.values_mut() {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new(output.output.name()).strong());

                                for i in 0..output.devices.len() {
                                    let device = &output.devices[i];

                                    let mut pick_state = PickState::Picked;

                                    ui.add(DeviceSelector::new(
                                        device.info().name(),
                                        &mut pick_state,
                                    ));

                                    match pick_state {
                                        PickState::Remove => {
                                            output.remove(i);
                                            break;
                                        }
                                        PickState::Pick(device) => {
                                            output.set(i, device);
                                        }
                                        _ => {}
                                    }
                                }

                                if output.can_add() {
                                    let mut pick_state = PickState::None;

                                    ui.add(DeviceSelector::new("Select...", &mut pick_state));

                                    if let PickState::Pick(device) = pick_state {
                                        output.add(device);
                                    }
                                }
                            });
                        });
                    }
                });
            });

            ui.add_space(5.0);
        });
    }
}
