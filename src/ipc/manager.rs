use crate::sync::Spinlock;
use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

struct Port {
    name: String,
    queue: Spinlock<VecDeque<Vec<u8>>>,
    capacity: usize,
}

static PORT_REGISTRY: Spinlock<Option<BTreeMap<String, Arc<Port>>>> = Spinlock::new(None);
static PORT_HANDLES: Spinlock<Vec<Arc<Port>>> = Spinlock::new(Vec::new());

fn scan_handles(port: &Arc<Port>) -> Option<usize> {
    let handles = PORT_HANDLES.lock();
    for (i, p) in handles.iter().enumerate() {
        if Arc::ptr_eq(p, port) {
            return Some(i);
        }
    }
    None
}

fn add_handle(port: Arc<Port>) -> usize {
    let mut handles = PORT_HANDLES.lock();
    // Reutilizar slots? Por enquanto append.
    handles.push(port);
    handles.len() - 1
}

fn get_port_by_handle(handle: usize) -> Option<Arc<Port>> {
    let handles = PORT_HANDLES.lock();
    handles.get(handle).cloned()
}

pub fn create_port(name: &str, capacity: usize) -> Result<usize, ()> {
    // Lazily init registry
    let mut registry_guard = PORT_REGISTRY.lock();
    if registry_guard.is_none() {
        *registry_guard = Some(BTreeMap::new());
    }
    let registry = registry_guard.as_mut().unwrap();

    if registry.contains_key(name) {
        return Err(()); // Already exists
    }

    let port = Arc::new(Port {
        name: String::from(name),
        queue: Spinlock::new(VecDeque::with_capacity(capacity)),
        capacity,
    });

    registry.insert(String::from(name), port.clone());
    drop(registry_guard); // liberar lock do map

    Ok(add_handle(port))
}

pub fn connect_port(name: &str) -> Result<usize, ()> {
    let mut registry_guard = PORT_REGISTRY.lock();
    if registry_guard.is_none() {
        *registry_guard = Some(BTreeMap::new());
    }
    let registry = registry_guard.as_mut().unwrap();

    if let Some(port) = registry.get(name) {
        Ok(add_handle(port.clone()))
    } else {
        Err(()) // Not found
    }
}

pub fn send_msg(handle: usize, data: &[u8]) -> Result<usize, ()> {
    if let Some(port) = get_port_by_handle(handle) {
        let mut queue = port.queue.lock();
        if queue.len() >= port.capacity {
            return Err(()); // Full (TODO: Block)
        }
        queue.push_back(Vec::from(data));
        Ok(data.len())
    } else {
        Err(())
    }
}

pub fn recv_msg(handle: usize, buf: &mut [u8]) -> Result<usize, ()> {
    if let Some(port) = get_port_by_handle(handle) {
        let mut queue = port.queue.lock();
        if let Some(msg) = queue.pop_front() {
            let len = core::cmp::min(buf.len(), msg.len());
            buf[..len].copy_from_slice(&msg[..len]);
            Ok(len)
        } else {
            // Empty (TODO: Block)
            Ok(0)
        }
    } else {
        Err(())
    }
}
