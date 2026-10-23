//! Owned, serialized access to the Zig probe, counter runtime, and packet simulator.
#![deny(unsafe_op_in_unsafe_fn)]
pub mod network;
pub mod simulation;

use std::{
    fmt,
    marker::PhantomData,
    rc::Rc,
    sync::{Mutex, MutexGuard},
};

pub const ABI_VERSION: u32 = 1;
pub const PAYLOAD_LIMIT: usize = 64;
pub const BATCH_LIMIT: usize = 16;
pub const ZIG_VERSION: &str = env!("WIREBOX_ZIG_VERSION");
static API: Mutex<()> = Mutex::new(());

fn api_lock() -> MutexGuard<'static, ()> {
    API.lock().unwrap_or_else(|poison| poison.into_inner())
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawEvent {
    id: u64,
    kind: u32,
    length: u32,
    bytes: [u8; PAYLOAD_LIMIT],
}

impl Default for RawEvent {
    fn default() -> Self {
        Self {
            id: 0,
            kind: 0,
            length: 0,
            bytes: [0; PAYLOAD_LIMIT],
        }
    }
}

unsafe extern "C" {
    fn wb_abi_version() -> u32;
    fn wb_create(version: u32, out: *mut u64) -> u32;
    fn wb_destroy(handle: u64) -> u32;
    fn wb_probe_submit(handle: u64, data: *const u8, length: usize) -> u32;
    fn wb_probe_read(handle: u64, out: *mut RawEvent, capacity: usize, count: *mut usize) -> u32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidArgument,
    AbiMismatch,
    InvalidHandle,
    Capacity,
    NotFound,
    Conflict,
    PastTime,
    Overflow,
    State,
    Allocation,
    InvalidPacket,
    Unsupported,
    InvalidResponse(u32),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "kernel: {self:?}")
    }
}
impl std::error::Error for Error {}

fn status(code: u32) -> Result<(), Error> {
    match code {
        0 => Ok(()),
        1 => Err(Error::InvalidArgument),
        2 => Err(Error::AbiMismatch),
        3 => Err(Error::InvalidHandle),
        4 => Err(Error::Capacity),
        5 => Err(Error::NotFound),
        6 => Err(Error::Conflict),
        7 => Err(Error::PastTime),
        8 => Err(Error::Overflow),
        9 => Err(Error::State),
        10 => Err(Error::Allocation),
        11 => Err(Error::InvalidPacket),
        12 => Err(Error::Unsupported),
        value => Err(Error::InvalidResponse(value)),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProbeEvent {
    pub id: u64,
    pub kind: u32,
    pub bytes: Vec<u8>,
}

/// Thread-bound owner of a Zig session. All views returned by this API are copies.
/// A session is deliberately !Send and !Sync; separate sessions are serialized too.
///
/// ```compile_fail
/// let session = wirebox_kernel::Session::new().unwrap();
/// std::thread::spawn(move || drop(session));
/// ```
pub struct Session {
    handle: Option<u64>,
    _thread_bound: PhantomData<Rc<()>>,
}

impl Session {
    pub fn new() -> Result<Self, Error> {
        let _guard = api_lock();
        // SAFETY: pure ABI query; create gets a valid writable output under the lock.
        if unsafe { wb_abi_version() } != ABI_VERSION {
            return Err(Error::AbiMismatch);
        }
        let mut handle = 0;
        status(unsafe { wb_create(ABI_VERSION, &mut handle) })?;
        Ok(Self {
            handle: Some(handle),
            _thread_bound: PhantomData,
        })
    }

    pub fn submit_probe(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let _guard = api_lock();
        // SAFETY: slice remains valid during the synchronous call; Zig copies it.
        status(unsafe { wb_probe_submit(self.handle.unwrap(), bytes.as_ptr(), bytes.len()) })
    }

    pub fn read_probe(&mut self, capacity: usize) -> Result<Vec<ProbeEvent>, Error> {
        if capacity > BATCH_LIMIT {
            return Err(Error::InvalidArgument);
        }
        let _guard = api_lock();
        let mut raw = vec![RawEvent::default(); capacity];
        let mut count = 0;
        // SAFETY: output points to `capacity` initialized aligned records; count is writable.
        status(unsafe {
            wb_probe_read(self.handle.unwrap(), raw.as_mut_ptr(), capacity, &mut count)
        })?;
        if count > capacity {
            return Err(Error::InvalidResponse(0));
        }
        raw[..count]
            .iter()
            .map(|event| {
                if event.length as usize > PAYLOAD_LIMIT {
                    return Err(Error::InvalidResponse(event.length));
                }
                Ok(ProbeEvent {
                    id: event.id,
                    kind: event.kind,
                    bytes: event.bytes[..event.length as usize].to_vec(),
                })
            })
            .collect()
    }

    pub fn close(mut self) -> Result<(), Error> {
        self.close_inner()
    }

    fn close_inner(&mut self) -> Result<(), Error> {
        if let Some(handle) = self.handle.take() {
            let _guard = api_lock();
            // SAFETY: only the owning Session can destroy this handle; Drop cannot repeat it.
            status(unsafe { wb_destroy(handle) })?;
        }
        Ok(())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.close_inner();
    }
}

/// Shared input fixture for CLI and UI; these are probe bytes, not a simulated packet.
pub const PROBE_BYTES: &[u8] = b"Wirebox / Zig to Rust / ABI v1";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_matches_c_and_zig() {
        assert_eq!(std::mem::size_of::<RawEvent>(), 80);
        assert_eq!(std::mem::align_of::<RawEvent>(), 8);
        assert_eq!(std::mem::offset_of!(RawEvent, bytes), 16);
    }

    #[test]
    fn copied_data_survives_mutation_and_owner_shutdown() {
        let mut session = Session::new().unwrap();
        let mut input = vec![0x12, 0x34];
        session.submit_probe(&input).unwrap();
        input.fill(0xff);
        let events = session.read_probe(1).unwrap();
        session.close().unwrap();
        assert_eq!(events[0].bytes, [0x12, 0x34]);
    }

    #[test]
    fn partial_batches_preserve_order_and_failed_submit_does_not_consume_ids() {
        let mut session = Session::new().unwrap();
        for n in 0..BATCH_LIMIT {
            session.submit_probe(&[n as u8]).unwrap();
        }
        assert_eq!(session.submit_probe(&[99]), Err(Error::Capacity));
        assert!(session.read_probe(0).unwrap().is_empty());
        let first = session.read_probe(3).unwrap();
        assert_eq!(first.iter().map(|e| e.id).collect::<Vec<_>>(), [1, 2, 3]);
        session.submit_probe(&[99]).unwrap();
        let rest = session.read_probe(BATCH_LIMIT).unwrap();
        assert_eq!(rest.len(), 14);
        assert_eq!(rest.last().unwrap().id, 17);
        assert_eq!(rest.last().unwrap().bytes, [99]);
        assert!(session.read_probe(1).unwrap().is_empty());
    }

    #[test]
    fn invalid_inputs_do_not_change_queue() {
        let mut session = Session::new().unwrap();
        assert_eq!(session.submit_probe(&[]), Err(Error::InvalidArgument));
        assert_eq!(session.submit_probe(&[1; 65]), Err(Error::InvalidArgument));
        assert_eq!(session.read_probe(17), Err(Error::InvalidArgument));
        session.submit_probe(&[42]).unwrap();
        assert_eq!(session.read_probe(1).unwrap()[0].id, 1);
    }

    #[test]
    fn raw_boundary_rejects_null_mismatched_abi_and_stale_handles() {
        let _guard = api_lock();
        // SAFETY: nulls are explicitly tested inputs; all non-null outputs are valid.
        unsafe {
            let mut handle = 123;
            assert_eq!(wb_create(ABI_VERSION, std::ptr::null_mut()), 1);
            assert_eq!(wb_create(999, &mut handle), 2);
            assert_eq!(handle, 0);
            assert_eq!(wb_create(ABI_VERSION, &mut handle), 0);
            let mut count = 9;
            assert_eq!(wb_probe_submit(handle, std::ptr::null(), 1), 1);
            assert_eq!(
                wb_probe_read(handle, std::ptr::null_mut(), 1, &mut count),
                1
            );
            assert_eq!(count, 0);
            assert_eq!(wb_destroy(handle), 0);
            let mut replacement = 0;
            assert_eq!(wb_create(ABI_VERSION, &mut replacement), 0);
            assert_ne!(handle, replacement);
            assert_eq!(wb_destroy(handle), 3);
            assert_eq!(wb_probe_submit(handle, [1].as_ptr(), 1), 3);
            assert_eq!(
                wb_probe_read(handle, std::ptr::null_mut(), 0, &mut count),
                3
            );
            assert_eq!(wb_destroy(replacement), 0);
        }
    }
}
