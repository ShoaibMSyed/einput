use std::collections::HashMap;

use eframe::egui::Ui;

use crate::FilterableConfig;

use super::Configure;

#[derive(Default)]
pub struct LoadTab {
    configs: HashMap<String, FilterableConfig>,
}

impl Configure {
    pub fn select_tab_load(&mut self) {
        self.tab_load.configs = self.configs.lock().unwrap().all.clone();
    }

    pub fn tab_load(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for (name, fcfg) in &self.tab_load.configs {
                if !fcfg.filter(&self.device) {
                    continue;
                }

                if ui.button(name).clicked() {
                    self.input_config = fcfg.config.clone();
                    self.update_config();
                }
            }
        });
    }
}
