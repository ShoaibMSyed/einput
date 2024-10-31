use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use einput_device::{DeviceId, DeviceInfo, DeviceInput};
use einput_util::shared::{Reader, Writer};

pub type DeviceReader = Reader<DeviceId, DeviceInput>;
pub type DeviceWriter = Writer<DeviceId, DeviceInput>;

pub struct DeviceTransformer {
    func: Box<dyn FnMut(&mut DeviceInput) + Send + Sync>,
    provider: Arc<dyn Fn() -> Box<dyn FnMut(&mut DeviceInput) + Send + Sync> + Send + Sync + 'static>,
}

impl DeviceTransformer {
    pub fn new<F: Fn() -> Box<dyn FnMut(&mut DeviceInput) + Send + Sync> + Send + Sync + 'static>(provider: F) -> Self {
        let func = provider();

        DeviceTransformer {
            func,
            provider: Arc::new(provider),
        }
    }

    fn call(&mut self, input: &mut DeviceInput) {
        (self.func)(input)
    }
}

impl Clone for DeviceTransformer {
    fn clone(&self) -> Self {
        let func = (self.provider)();
        Self { func, provider: self.provider.clone() }
    }
}

impl Default for DeviceTransformer {
    fn default() -> Self {
        Self::new(|| Box::new(|_| {}))
    }
}

#[derive(Clone)]
pub struct Device {
    info: Arc<Mutex<DeviceInfo>>,

    owned: Arc<AtomicBool>,

    pub(crate) transformer: Arc<Mutex<DeviceTransformer>>,

    input_writer: DeviceWriter,
    input_writer_raw: DeviceWriter,
}

impl Device {
    pub(crate) fn new(info: DeviceInfo, transformer: DeviceTransformer) -> Self {
        let transformer: Arc<Mutex<DeviceTransformer>> = Arc::new(Mutex::new(transformer));
        let input_writer = Writer::new();
        let input_writer_raw = Writer::new();

        Device {
            info: Arc::new(Mutex::new(info)),

            owned: Arc::new(AtomicBool::new(false)),

            transformer,

            input_writer,
            input_writer_raw,
        }
    }

    pub(crate) fn replace(&self, info: DeviceInfo) -> Option<DeviceOwner> {
        let mut owner = self.create_owner()?;

        let mut self_info = self.info.lock().unwrap();

        if self_info.input == info.input && self_info.output == info.output {
            return Some(owner);
        }

        *self_info = info.clone();
        owner.input = DeviceInput::new(&self_info.input);
        owner.input_raw = owner.input.clone();

        Some(owner)
    }

    pub(crate) fn create_owner(&self) -> Option<DeviceOwner> {
        if self.owned.swap(true, Ordering::Relaxed) {
            return None;
        }

        let self_info = self.info.lock().unwrap();

        let input = DeviceInput::new(&self_info.input);

        Some(DeviceOwner {
            input_raw: input.clone(),
            input,
            id: self_info.id().clone(),
            
            transformer: self.transformer.clone(),

            owned: self.owned.clone(),

            writer: self.input_writer.clone(),
            writer_raw: self.input_writer_raw.clone(),
        })
    }

    pub fn owned(&self) -> bool {
        self.owned.load(Ordering::Relaxed)
    }

    pub fn info(&self) -> DeviceInfo {
        self.info.lock().unwrap().clone()
    }

    pub fn register_reader(&self, reader: &mut DeviceReader) {
        self.input_writer.register(reader);
    }

    pub fn register_reader_raw(&self, reader: &mut DeviceReader) {
        self.input_writer_raw.register(reader);
    }
}

pub struct DeviceOwner {
    input: DeviceInput,
    input_raw: DeviceInput,
    id: DeviceId,

    transformer: Arc<Mutex<DeviceTransformer>>,

    owned: Arc<AtomicBool>,

    writer: Writer<DeviceId, DeviceInput>,
    writer_raw: Writer<DeviceId, DeviceInput>,
}

impl DeviceOwner {
    pub fn update(&mut self, f: impl FnOnce(&mut DeviceInput)) {
        f(&mut self.input_raw);
        self.writer_raw.write(&self.id, &self.input_raw);

        self.input.clone_from(&self.input_raw);
        self
            .transformer
            .lock()
            .expect("device transformer poisoned")
            .call(&mut self.input);
        self.writer.write(&self.id, &self.input);
    }
}

impl Drop for DeviceOwner {
    fn drop(&mut self) {
        self.owned.store(false, Ordering::Relaxed);
    }
}
