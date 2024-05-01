use std::{
    collections::HashMap,
    hash::Hash,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    }, time::Duration,
};

pub struct Writer<K, V>(Arc<Mutex<InnerWriter<K, V>>>);

impl<K, V> Clone for Writer<K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<K, V> Default for Writer<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Writer<K, V> {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(InnerWriter {
            readers: Vec::new(),
        })))
    }
}

impl<K: Clone, V: Clone> Writer<K, V> {
    pub fn register(&self, reader: &mut Reader<K, V>) {
        let mut lock = self.0.lock().expect("Writer is poisoned");
        lock.readers.push(reader.inner.clone());
    }

    pub fn write(&self, key: &K, value: &V)
    where
        K: Hash + Eq,
    {
        let Ok(mut lock) = self.0.lock() else { return };

        lock.readers.retain(|reader| {
            let Ok(mut lock) = reader.value.lock() else {
                return true;
            };

            match lock.get_mut(key) {
                Some(old) => {
                    old.clone_from(value);
                }
                None => {
                    lock.insert(key.clone(), value.clone());
                }
            }

            reader.cvar.notify_all();

            reader.owned.load(Ordering::Relaxed)
        });
    }
}

pub struct Reader<K, V> {
    inner: Arc<InnerReader<K, V>>,
    map: HashMap<K, V>,
}

impl<K, V> Reader<K, V> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InnerReader {
                owned: AtomicBool::new(true),
                value: Mutex::default(),
                cvar: Condvar::new(),
            }),
            map: HashMap::default(),
        }
    }

    pub fn current(&self) -> &HashMap<K, V> {
        &self.map
    }
}

impl<K: Clone, V: Clone> Reader<K, V> {
    pub fn update(&mut self) -> &HashMap<K, V> {
        self.map
            .clone_from(&self.inner.value.lock().expect("Reader was poisoned"));
        &self.map
    }

    pub fn wait(&mut self) -> &HashMap<K, V> {
        let guard = self.inner.value.lock().expect("Reader was poisoned");
        let guard = self.inner.cvar.wait(guard).expect("Reader was poisoned");
        self.map.clone_from(&guard);
        drop(guard);

        &self.map
    }

    pub fn wait_timeout(&mut self, dur: Duration) -> Option<&HashMap<K, V>> {
        let guard = self.inner.value.lock().expect("Reader was poisoned");
        let (guard, wait) = self.inner.cvar.wait_timeout(guard, dur).expect("Reader was poisoned");
        self.map.clone_from(&guard);
        drop(guard);

        if wait.timed_out() {
            return None;
        }

        Some(&self.map)
    }
}

impl<K, V> Default for Reader<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Drop for Reader<K, V> {
    fn drop(&mut self) {
        self.inner.owned.store(false, Ordering::Relaxed);
    }
}

struct InnerWriter<K, V> {
    readers: Vec<Arc<InnerReader<K, V>>>,
}

struct InnerReader<K, V> {
    owned: AtomicBool,
    value: Mutex<HashMap<K, V>>,
    cvar: Condvar,
}
