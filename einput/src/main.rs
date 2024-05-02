#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use eframe::{
    egui::{Context, Id, ViewportBuilder, ViewportCommand, ViewportId},
    CreationContext, NativeOptions,
};
use einput_core::{
    device::{Device, DeviceReader},
    output::Output,
    EInput,
};
use einput_device::{input::DeviceInputConfig, DeviceId};
use log::error;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;

use self::configure::Configure;

mod configure;
mod devices;
mod outputs;
mod widgets;

fn main() {
    SimpleLogger::new()
            .with_level(log::LevelFilter::Debug)
            .init()
            .unwrap();

    let native_options = NativeOptions {
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };

    let result = eframe::run_native(
        "einput",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    );

    match result {
        Ok(()) => {}
        Err(e) => {
            error!("eframe error: {e}");
        }
    }
}

struct App {
    einput: EInput,
    last_refresh: Instant,
    tracking: HashMap<DeviceId, Device>,
    tracking_order: Vec<DeviceId>,
    reader: DeviceReader,
    outputs: HashMap<String, OutputData>,

    configuring: Vec<ConfigureState>,

    configs: Arc<Mutex<Configs>>,
}

impl App {
    fn new(cc: &CreationContext) -> Self {
        let storage = cc.storage.unwrap();
        let configs: Configs =
            match serde_json::from_str(storage.get_string("configs").as_deref().unwrap_or("")) {
                Ok(configs) => configs,
                Err(e) => {
                    error!("error loading Configs: {e}");
                    Configs::default()
                }
            };

        let einput = EInput::new();
        configs.set_to_last(&einput);
        einput_driver_gc::start(einput.clone());

        let mut outputs: HashMap<String, OutputData> = outputs::all()
            .into_iter()
            .map(|(id, out)| (id, OutputData::new(out)))
            .collect();

        match serde_json::from_str::<'_, Preset>(
            storage.get_string("preset").as_deref().unwrap_or(""),
        ) {
            Ok(preset) => {
                for (output_id, device_id) in preset.output_map {
                    let Some(output) = outputs.get_mut(&output_id) else {
                        continue;
                    };

                    output.devices = device_id.into_iter().map(|id| einput.get_or_create(id)).collect();
                    output.output.update(&output.devices);
                }
            }
            Err(e) => {
                error!("error loading Preset: {e}");
            }
        }

        App {
            outputs,
            einput,
            last_refresh: Instant::now(),
            tracking: HashMap::new(),
            tracking_order: Vec::new(),
            reader: DeviceReader::new(),
            configuring: Vec::new(),
            configs: Arc::new(Mutex::new(configs)),
        }
    }

    fn refresh(&mut self) {
        self.reader.update();

        if Instant::now() - self.last_refresh <= Duration::from_secs(1) {
            return;
        }

        for device in self.einput.devices() {
            let id = device.info().id().clone();
            if !self.tracking.contains_key(&id) {
                self.tracking
                    .insert(id.clone(), device.clone());
                self.tracking_order.push(id.clone());
                device.register_reader(&mut self.reader);
            }
        }

        self.tracking_order.sort_by_cached_key(|id| self.tracking.get(id).unwrap().info().name().to_owned());

        self.last_refresh = Instant::now();
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        self.refresh();

        self.bottom_panel(ctx);
        self.central_panel(ctx);

        let mut i = 0;
        self.configuring.retain(|state| {
            let configure = state.configure.clone();
            let close = state.close.clone();

            ctx.show_viewport_deferred(
                ViewportId(Id::new("configure_window").with(i)),
                ViewportBuilder::default()
                    .with_title(&state.title)
                    .with_inner_size([600.0, 300.0]),
                move |ctx, _| {
                    let mut lock = configure.lock().expect("window poisoned");
                    lock.show(ctx);

                    if ctx.input(|i| i.viewport().close_requested()) {
                        lock.update_config();
                        close.store(true, Ordering::Relaxed);
                        ctx.send_viewport_cmd_to(ctx.parent_viewport_id(), ViewportCommand::Focus);
                    }
                },
            );

            i += 1;

            !state.close.load(Ordering::Relaxed)
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let mut configs = self.configs.lock().unwrap();
        configs.update_last_from_connected(&self.einput);

        match serde_json::to_string::<Configs>(&configs) {
            Ok(string) => {
                drop(configs);
                storage.set_string("configs", string);
            }
            Err(e) => {
                error!("error serializing Configs: {e}");
            }
        };

        let output_map = self
            .outputs
            .iter()
            .map(|(id, data)| {
                (
                    id.clone(),
                    data.devices
                        .iter()
                        .map(|dev| dev.info().id().clone())
                        .collect(),
                )
            })
            .collect();

        let preset = Preset { output_map };

        match serde_json::to_string::<Preset>(&preset) {
            Ok(string) => {
                storage.set_string("preset", string);
            }
            Err(e) => {
                error!("error serializing Preset: {e}");
            }
        }
    }
}

struct OutputData {
    output: Box<dyn Output>,
    devices: Vec<Device>,
}

impl OutputData {
    fn new(output: Box<dyn Output>) -> Self {
        Self {
            output,
            devices: Vec::new(),
        }
    }

    fn can_add(&self) -> bool {
        self.devices.len() < self.output.max_devices()
    }

    fn remove(&mut self, index: usize) {
        self.devices.remove(index);
        self.output.update(&self.devices);
    }

    fn set(&mut self, index: usize, to: Device) {
        if self.devices[index].info().id() == to.info().id() {
            return;
        }

        self.devices[index] = to;
        self.output.update(&self.devices);
    }

    fn add(&mut self, device: Device) {
        self.devices.push(device);
        self.output.update(&self.devices);
    }
}

struct ConfigureState {
    configure: Arc<Mutex<Configure>>,
    close: Arc<AtomicBool>,
    title: String,
    id: DeviceId,
}

impl ConfigureState {
    fn new(einput: EInput, device: Device, configs: Arc<Mutex<Configs>>) -> Self {
        let close = Arc::new(AtomicBool::new(false));
        let title = format!("Configure {}", device.info().name());
        let id = device.info().id().clone();

        Self {
            configure: Arc::new(Mutex::new(Configure::new(einput, device, configs))),
            close,
            title,
            id,
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct Configs {
    last: HashMap<DeviceId, DeviceInputConfig>,
    all: HashMap<String, FilterableConfig>,
}

impl Configs {
    fn update_last(&mut self, id: &DeviceId, einput: &EInput) {
        let Some(config) = einput.get_input_config(id) else {
            return;
        };
        self.last.insert(id.clone(), config);
    }

    fn update_last_from_connected(&mut self, einput: &EInput) {
        for device in einput.devices() {
            let id = device.info().id().clone();
            self.update_last(&id, einput);
        }
    }

    fn set_to_last(&self, einput: &EInput) {
        for (id, config) in &self.last {
            einput.set_input_config(id.clone(), config.clone());
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct FilterableConfig {
    filter: ConfigFilter,
    config: DeviceInputConfig,
}

impl FilterableConfig {
    fn no_filter(config: &DeviceInputConfig) -> Self {
        FilterableConfig {
            filter: ConfigFilter::None,
            config: config.clone(),
        }
    }

    fn product(device: &Device, config: &DeviceInputConfig) -> Self {
        FilterableConfig {
            filter: ConfigFilter::Product(device.info().product_name().to_owned()),
            config: config.clone(),
        }
    }

    fn id(device: &Device, config: &DeviceInputConfig) -> Self {
        FilterableConfig {
            filter: ConfigFilter::Id(device.info().id().clone()),
            config: config.clone(),
        }
    }

    fn filter(&self, device: &Device) -> bool {
        match &self.filter {
            ConfigFilter::None => true,
            ConfigFilter::Product(p) => device.info().product_name() == p,
            ConfigFilter::Id(id) => device.info().id() == id,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
enum ConfigFilter {
    None,
    Product(String),
    Id(DeviceId),
}

#[derive(Clone, Serialize, Deserialize)]
struct Preset {
    output_map: HashMap<String, Vec<DeviceId>>,
}
