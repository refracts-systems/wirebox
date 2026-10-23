//! Owned copies of the Zig runtime. Entity/timer IDs are scoped to one World.
use super::{Error, api_lock, status};
use std::{fmt, marker::PhantomData, rc::Rc};

pub const SIM_VERSION: u32 = 1;
pub const ENTITY_LIMIT: usize = 128;
pub const TRACE_LIMIT: usize = 256;

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum DeviceRole {
    Host = 1,
    Switch = 2,
    Router = 3,
}
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum Action {
    AddCounter = 1,
    RandomAddCounter = 2,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Entity {
    pub id: u64,
    pub owner: u64,
    pub a: u64,
    pub b: u64,
    pub counter: u64,
    pub kind: u32,
    pub role: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Event {
    pub id: u64,
    pub time_ns: u64,
    pub target_id: u64,
    pub timer_id: u64,
    pub cause_id: u64,
    pub amount: u64,
    pub action: u32,
    reserved: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Trace {
    pub event: Event,
    pub ordinal: u64,
    pub before: u64,
    pub after: u64,
}
#[repr(C)]
struct RawSchedule {
    target_id: u64,
    time_ns: u64,
    interval_ns: u64,
    amount: u64,
    action: u32,
    reserved: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    pub now_ns: u64,
    pub processed: u64,
    /// 0 empty step, 1 horizon reached, 2 budget exhausted, 3 stepped, 4 error.
    pub reason: u32,
    reserved: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub now_ns: u64,
    pub seed: u64,
    pub rng_state: u64,
    pub next_event_id: u64,
    pub next_timer_id: u64,
    pub executed: u64,
    pub dropped: u64,
    pub last_event_id: u64,
    pub entity_count: u32,
    pub pending_count: u32,
    pub timer_count: u32,
    pub journal_count: u32,
}

unsafe extern "C" {
    fn wb_sim_version() -> u32;
    fn wb_sim_create(version: u32, seed: u64, retention: usize, out: *mut u64) -> u32;
    fn wb_sim_destroy(handle: u64) -> u32;
    fn wb_sim_add(handle: u64, kind: u32, role: u32, a: u64, b: u64, out: *mut u64) -> u32;
    fn wb_sim_remove(handle: u64, id: u64) -> u32;
    fn wb_sim_schedule(handle: u64, spec: *const RawSchedule, out: *mut u64) -> u32;
    fn wb_sim_cancel(handle: u64, id: u64) -> u32;
    fn wb_sim_step(handle: u64, out: *mut Progress) -> u32;
    fn wb_sim_run_until(handle: u64, horizon: u64, budget: u64, out: *mut Progress) -> u32;
    fn wb_sim_snapshot(handle: u64, out: *mut Snapshot) -> u32;
    fn wb_sim_entities(handle: u64, out: *mut Entity, capacity: usize, count: *mut usize) -> u32;
    fn wb_sim_trace(
        handle: u64,
        after: u64,
        out: *mut Trace,
        capacity: usize,
        count: *mut usize,
    ) -> u32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunError {
    pub error: Error,
    /// Prefix committed before a failing event; the failing event remains pending.
    pub progress: Progress,
}
impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} after {} events at {} ns",
            self.error, self.progress.processed, self.progress.now_ns
        )
    }
}
impl std::error::Error for RunError {}

/// Thread-bound owner. Calls are synchronous, bounded, serialized, and copy all data.
///
/// ```compile_fail
/// let world = wirebox_kernel::simulation::World::new(0, 16).unwrap();
/// std::thread::spawn(move || drop(world));
/// ```
pub struct World {
    handle: Option<u64>,
    _thread_bound: PhantomData<Rc<()>>,
}
impl World {
    pub fn new(seed: u64, retention: usize) -> Result<Self, Error> {
        let _guard = api_lock();
        // SAFETY: serialized pure query and valid scalar output, no pointer retained.
        if unsafe { wb_sim_version() } != SIM_VERSION {
            return Err(Error::AbiMismatch);
        }
        let mut handle = 0;
        status(unsafe { wb_sim_create(SIM_VERSION, seed, retention, &mut handle) })?;
        Ok(Self {
            handle: Some(handle),
            _thread_bound: PhantomData,
        })
    }
    fn add(&mut self, kind: u32, role: u32, a: u64, b: u64) -> Result<u64, Error> {
        let _guard = api_lock();
        let mut id = 0;
        // SAFETY: live owned handle, serialized mutation, valid writable scalar.
        status(unsafe { wb_sim_add(self.handle.unwrap(), kind, role, a, b, &mut id) })?;
        Ok(id)
    }
    pub fn add_device(&mut self, role: DeviceRole) -> Result<u64, Error> {
        self.add(1, role as u32, 0, 0)
    }
    pub fn add_port(&mut self, owner: u64) -> Result<u64, Error> {
        self.add(2, 0, owner, 0)
    }
    pub fn connect(&mut self, a: u64, b: u64) -> Result<u64, Error> {
        self.add(3, 0, a, b)
    }
    pub fn remove(&mut self, id: u64) -> Result<(), Error> {
        let _guard = api_lock();
        // SAFETY: live owned handle, scalar IDs are validated by Zig.
        status(unsafe { wb_sim_remove(self.handle.unwrap(), id) })
    }
    /// interval_ns=0 schedules once. RandomAddCounter samples [0, amount).
    pub fn schedule(
        &mut self,
        target_id: u64,
        time_ns: u64,
        interval_ns: u64,
        action: Action,
        amount: u64,
    ) -> Result<u64, Error> {
        let spec = RawSchedule {
            target_id,
            time_ns,
            interval_ns,
            amount,
            action: action as u32,
            reserved: 0,
        };
        let _guard = api_lock();
        let mut timer = 0;
        // SAFETY: aligned input/output live for this synchronous copy call.
        status(unsafe { wb_sim_schedule(self.handle.unwrap(), &spec, &mut timer) })?;
        Ok(timer)
    }
    pub fn cancel(&mut self, timer_id: u64) -> Result<(), Error> {
        let _guard = api_lock();
        // SAFETY: serialized call with live handle and validated scalar ID.
        status(unsafe { wb_sim_cancel(self.handle.unwrap(), timer_id) })
    }
    pub fn step(&mut self) -> Result<Progress, RunError> {
        let _guard = api_lock();
        let mut progress = Progress::default();
        // SAFETY: exclusive world access and valid initialized output.
        let code = unsafe { wb_sim_step(self.handle.unwrap(), &mut progress) };
        status(code).map_err(|error| RunError { error, progress })?;
        Ok(progress)
    }
    pub fn run_until(&mut self, horizon: u64, budget: u64) -> Result<Progress, RunError> {
        let _guard = api_lock();
        let mut progress = Progress::default();
        // SAFETY: exclusive world access and valid initialized output.
        let code =
            unsafe { wb_sim_run_until(self.handle.unwrap(), horizon, budget, &mut progress) };
        status(code).map_err(|error| RunError { error, progress })?;
        Ok(progress)
    }
    pub fn snapshot(&self) -> Result<Snapshot, Error> {
        let _guard = api_lock();
        let mut snapshot = Snapshot::default();
        // SAFETY: valid initialized output, no pointer retained.
        status(unsafe { wb_sim_snapshot(self.handle.unwrap(), &mut snapshot) })?;
        Ok(snapshot)
    }
    /// Returns a complete copy sorted by entity ID.
    pub fn entities(&self) -> Result<Vec<Entity>, Error> {
        let _guard = api_lock();
        let mut out = vec![Entity::default(); ENTITY_LIMIT];
        let mut count = 0;
        // SAFETY: allocated initialized records for the entire advertised capacity.
        status(unsafe {
            wb_sim_entities(
                self.handle.unwrap(),
                out.as_mut_ptr(),
                out.len(),
                &mut count,
            )
        })?;
        if count > out.len() {
            return Err(Error::InvalidResponse(0));
        }
        out.truncate(count);
        out.sort_by_key(|entity| entity.id);
        Ok(out)
    }
    /// Copy executions after an ordinal; use snapshot.dropped to detect retention loss.
    pub fn trace(&self, after_ordinal: u64, capacity: usize) -> Result<Vec<Trace>, Error> {
        if capacity > TRACE_LIMIT {
            return Err(Error::InvalidArgument);
        }
        let _guard = api_lock();
        let mut out = vec![Trace::default(); capacity];
        let mut count = 0;
        // SAFETY: allocated initialized records; capacity zero is accepted by Zig.
        status(unsafe {
            wb_sim_trace(
                self.handle.unwrap(),
                after_ordinal,
                out.as_mut_ptr(),
                capacity,
                &mut count,
            )
        })?;
        if count > capacity {
            return Err(Error::InvalidResponse(0));
        }
        out.truncate(count);
        Ok(out)
    }
    pub fn close(mut self) -> Result<(), Error> {
        self.close_inner()
    }
    fn close_inner(&mut self) -> Result<(), Error> {
        if let Some(handle) = self.handle.take() {
            let _guard = api_lock();
            // SAFETY: owned handle is destroyed once; no outstanding Zig views exist.
            status(unsafe { wb_sim_destroy(handle) })?;
        }
        Ok(())
    }
}
impl Drop for World {
    fn drop(&mut self) {
        let _ = self.close_inner();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn c_layouts_match() {
        assert_eq!(std::mem::size_of::<Entity>(), 48);
        assert_eq!(std::mem::size_of::<Event>(), 56);
        assert_eq!(std::mem::size_of::<Trace>(), 80);
        assert_eq!(std::mem::offset_of!(Trace, ordinal), 56);
        assert_eq!(std::mem::size_of::<RawSchedule>(), 40);
        assert_eq!(std::mem::size_of::<Progress>(), 24);
        assert_eq!(std::mem::size_of::<Snapshot>(), 80);
        assert_eq!(std::mem::offset_of!(Snapshot, entity_count), 64);
    }
    #[test]
    fn runtime_ffi_copies_survive_close_and_trace_cursor_is_execution_ordinal() {
        let mut world = World::new(0, 16).unwrap();
        let id = world.add_device(DeviceRole::Host).unwrap();
        world.schedule(id, 20, 0, Action::AddCounter, 5).unwrap();
        world.schedule(id, 10, 0, Action::AddCounter, 3).unwrap();
        let run = world.run_until(30, 10).unwrap();
        assert_eq!((run.processed, run.now_ns, run.reason), (2, 30, 1));
        assert!(world.trace(0, 0).unwrap().is_empty());
        let first = world.trace(0, 1).unwrap();
        let second = world.trace(first[0].ordinal, 1).unwrap();
        assert_eq!((first[0].event.id, second[0].event.id), (2, 1));
        let state = world.entities().unwrap();
        world.remove(id).unwrap();
        world.close().unwrap();
        assert_eq!(state[0].counter, 8);
        assert_eq!((first[0].before, first[0].after), (0, 3));
        assert_eq!((second[0].before, second[0].after), (3, 8));
    }
    #[test]
    fn graph_lifecycle_and_periodic_budget_through_ffi() {
        let mut world = World::new(9, 1).unwrap();
        let a = world.add_device(DeviceRole::Router).unwrap();
        let b = world.add_device(DeviceRole::Host).unwrap();
        let pa = world.add_port(a).unwrap();
        let pb = world.add_port(b).unwrap();
        let link = world.connect(pa, pb).unwrap();
        assert_eq!(world.connect(pb, pa), Err(Error::Conflict));
        world.schedule(link, 1, 1, Action::AddCounter, 1).unwrap();
        let run = world.run_until(100, 3).unwrap();
        assert_eq!((run.reason, run.now_ns), (2, 3));
        world.remove(a).unwrap();
        let state = world.snapshot().unwrap();
        assert_eq!((state.entity_count, state.pending_count), (2, 0));
        assert_eq!((state.executed, state.dropped), (3, 2));
        assert_eq!(world.step().unwrap().now_ns, 3);
        assert_eq!(world.run_until(100, 0).unwrap().now_ns, 100);
        assert_eq!(world.add_port(a), Err(Error::NotFound));
        assert_eq!(world.run_until(99, 1).unwrap_err().error, Error::PastTime);
    }
    #[test]
    fn run_error_preserves_prefix_and_failed_event_can_be_cancelled() {
        let mut world = World::new(0, 256).unwrap();
        let id = world.add_device(DeviceRole::Host).unwrap();
        world
            .schedule(id, 1, 0, Action::AddCounter, u64::MAX)
            .unwrap();
        let bad = world.schedule(id, 2, 0, Action::AddCounter, 1).unwrap();
        let error = world.run_until(10, 10).unwrap_err();
        assert_eq!(error.error, Error::Overflow);
        assert_eq!((error.progress.processed, error.progress.now_ns), (1, 1));
        assert_eq!(world.snapshot().unwrap().pending_count, 1);
        world.cancel(bad).unwrap();
        assert_eq!(world.run_until(10, 0).unwrap().reason, 1);
        assert_eq!(
            world.schedule(id, 10, 0, Action::RandomAddCounter, 0),
            Err(Error::InvalidArgument)
        );
        assert_eq!(world.trace(0, 257), Err(Error::InvalidArgument));
    }
    #[test]
    fn raw_runtime_handles_cannot_alias_probe_handles() {
        let _guard = api_lock();
        // SAFETY: all pointers valid, every successfully created handle is closed.
        unsafe {
            let mut probe = 0;
            let mut world = 0;
            assert_eq!(
                super::super::wb_create(super::super::ABI_VERSION, &mut probe),
                0
            );
            assert_eq!(wb_sim_create(1, 0, 0, &mut world), 0);
            assert_eq!(wb_sim_destroy(probe), 3);
            assert_eq!(super::super::wb_destroy(world), 3);
            assert_eq!(wb_sim_destroy(world), 0);
            assert_eq!(super::super::wb_destroy(probe), 0);
        }
    }
}
