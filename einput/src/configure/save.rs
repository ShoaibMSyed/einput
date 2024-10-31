use std::collections::HashMap;

use eframe::egui::{ScrollArea, Ui};

use crate::FilterableConfig;

use super::Configure;

#[derive(Default)]
pub struct SaveTab {
    name: String,
    filter: Filter,
    configs: HashMap<String, FilterableConfig>,
}

impl Configure {
    pub fn select_tab_save(&mut self) {
        self.tab_save.configs = self.configs.lock().unwrap().all.clone();
    }
    
    pub fn tab_save(&mut self, ui: &mut Ui) {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Config Name");
            ui.text_edit_singleline(&mut self.tab_save.name);

            if ui.button("Save").clicked() {
                let fcfg = match self.tab_save.filter {
                    Filter::None => FilterableConfig::no_filter(&self.config),
                    Filter::Product => FilterableConfig::product(&self.device, &self.config),
                    Filter::Id => FilterableConfig::id(&self.device, &self.config),
                };
                self.configs.lock().unwrap().all.insert(std::mem::take(&mut self.tab_save.name), fcfg);
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Save For");
            ui.radio_value(&mut self.tab_save.filter, Filter::Id, "This Device");
            ui.radio_value(&mut self.tab_save.filter, Filter::Product, format!("Any {}", self.device.info().product_name()));
            ui.radio_value(&mut self.tab_save.filter, Filter::None, "Any Device");
        });

        ui.separator();

        ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for name in self.tab_save.configs.keys() {
                    let button = ui.button(name);

                    button.context_menu(|ui| {
                        if ui.button("Delete").clicked() {
                            self.configs.lock().unwrap().all.remove(name);
                            changed = true;
                        }
                    });

                    if button.clicked() {
                        if let Some(fcfg) = self.configs.lock().unwrap().all.get_mut(name) {
                            fcfg.config = self.config.clone();
                        }
                    }
                }
            });
        });

        if changed {
            self.select_tab_save();
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Filter {
    None,
    #[default]
    Product,
    Id,
}