use std::sync::{Arc, Mutex};

use eframe::egui::{Align, CentralPanel, Context, Layout, RichText, SidePanel, Ui};
use einput_config::DeviceConfig;
use einput_core::{
    device::{Device, DeviceReader},
    EInput,
};
use einput_device::DeviceInput;

use crate::Configs;

use self::{load::LoadTab, save::SaveTab, sticks::SticksTab};

mod buttons;
mod load;
mod overview;
mod save;
mod sticks;
mod triggers;

pub struct Configure {
    einput: EInput,
    device: Device,
    reader: DeviceReader,
    raw_reader: DeviceReader,

    config: DeviceConfig,
    configs: Arc<Mutex<Configs>>,

    tab: Tab,
    tab_load: LoadTab,
    tab_save: SaveTab,
    tab_sticks: SticksTab,
}

impl Configure {
    pub fn new(einput: EInput, device: Device, configs: Arc<Mutex<Configs>>) -> Self {
        let mut reader = DeviceReader::new();
        device.register_reader(&mut reader);

        let mut raw_reader = DeviceReader::new();
        device.register_reader_raw(&mut raw_reader);

        let config = configs.lock().unwrap().last.get(device.info().id()).cloned().unwrap_or_default();

        Configure {
            einput,
            device,
            reader,
            raw_reader,

            config,
            configs,

            tab: Tab::Overview,
            tab_load: LoadTab::default(),
            tab_save: SaveTab::default(),
            tab_sticks: SticksTab::default(),
        }
    }

    pub fn show(&mut self, ctx: &Context) {
        ctx.request_repaint();
        self.reader.update();
        self.raw_reader.update();

        SidePanel::left("configure_side_panel")
            .default_width(100.0)
            .show(ctx, |ui| {
                ui.with_layout(
                    Layout::top_down(Align::Min).with_cross_justify(true),
                    |ui| {
                        ui.add_space(5.0);

                        self.tab_select(ui, Tab::Overview);
                        ui.shrink_width_to_current();

                        ui.add_space(5.0);
                        ui.label(RichText::new("Input").strong());
                        ui.add_space(5.0);

                        self.tab_select(ui, Tab::Buttons);
                        self.tab_select(ui, Tab::Sticks);
                        self.tab_select(ui, Tab::Triggers);

                        ui.add_space(5.0);
                        ui.label(RichText::new("Config").strong());
                        ui.add_space(5.0);

                        self.tab_select(ui, Tab::Load);
                        self.tab_select(ui, Tab::Save);
                    },
                );
            });

        CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Overview => self.tab_overview(ui),

            Tab::Buttons => self.tab_buttons(ui),
            Tab::Sticks => self.tab_sticks(ui),
            Tab::Triggers => self.tab_triggers(ui),

            Tab::Load => self.tab_load(ui),
            Tab::Save => self.tab_save(ui),
        });
    }

    fn tab_select(&mut self, ui: &mut Ui, tab: Tab) {
        if ui.selectable_label(self.tab == tab, tab.name()).clicked() {
            self.tab = tab;

            if tab == Tab::Load {
                self.select_tab_load();
            } else if tab == Tab::Save {
                self.select_tab_save();
            }
        }
    }

    fn get_input(&self) -> Option<&DeviceInput> {
        self.reader.current().values().next()
    }

    fn get_raw_input(&self) -> Option<&DeviceInput> {
        self.raw_reader.current().values().next()
    }

    pub fn update_config(&self) {
        self.configs.lock().unwrap().update_device(self.device.info().id().clone(), self.config.clone(), &self.einput);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Overview,
    Buttons,
    Sticks,
    Triggers,
    Save,
    Load,
}

impl Tab {
    fn name(&self) -> &str {
        match self {
            Tab::Overview => "Overview",
            Tab::Buttons => "Buttons",
            Tab::Sticks => "Sticks",
            Tab::Triggers => "Triggers",
            Tab::Save => "Save",
            Tab::Load => "Load",
        }
    }
}
