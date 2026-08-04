//! Input-activity probe (counts only; never key content).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use crate::error::CollectorResult;

/// OS (or mock) source of key-down counts since last drain.
pub trait InputCountProbe: Send + Sync {
    /// Returns and clears the pending key-down count (aggregates only).
    fn take_count(&self) -> CollectorResult<u64>;

    /// Whether Accessibility (or equivalent) is available for live counting.
    /// Default: `true` for mocks; system probe reports real trust state.
    fn accessibility_trusted(&self) -> bool {
        true
    }
}

/// Test / scriptable counter: push deltas via [`ScriptedInputProbe::push`].
#[derive(Debug, Default)]
pub struct ScriptedInputProbe {
    pending: AtomicU64,
    trusted: AtomicBool,
}

impl ScriptedInputProbe {
    /// Creates a trusted probe with no pending counts.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending: AtomicU64::new(0),
            trusted: AtomicBool::new(true),
        }
    }

    /// Sets Accessibility-trust simulation (deny → graceful idle in production path).
    pub fn set_trusted(&self, trusted: bool) {
        self.trusted.store(trusted, Ordering::SeqCst);
    }

    /// Adds key-down events to the pending aggregate (never stores characters).
    pub fn push(&self, count: u64) {
        self.pending.fetch_add(count, Ordering::SeqCst);
    }
}

impl InputCountProbe for ScriptedInputProbe {
    fn take_count(&self) -> CollectorResult<u64> {
        if !self.trusted.load(Ordering::SeqCst) {
            return Ok(0);
        }
        Ok(self.pending.swap(0, Ordering::SeqCst))
    }

    fn accessibility_trusted(&self) -> bool {
        self.trusted.load(Ordering::SeqCst)
    }
}

/// Shared atomic counter used by the system event-tap callback.
#[derive(Debug, Default)]
pub struct SharedKeyCounter {
    count: AtomicU64,
}

impl SharedKeyCounter {
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            count: AtomicU64::new(0),
        })
    }

    pub fn record_key_down(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn take(&self) -> u64 {
        self.count.swap(0, Ordering::AcqRel)
    }
}

/// Production probe: listen-only key-down counter when Accessibility is trusted.
#[derive(Debug)]
pub struct SystemInputProbe {
    counter: Arc<SharedKeyCounter>,
    /// When false, tap was not started (denied / unsupported) — idle drain returns 0.
    listening: AtomicBool,
    #[cfg(target_os = "macos")]
    _tap_guard: Option<macos_tap::TapGuard>,
}

impl Default for SystemInputProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemInputProbe {
    /// Creates a probe and attempts to start a listen-only event tap on macOS.
    /// Failure / missing Accessibility → no panic; [`take_count`] returns 0.
    #[must_use]
    pub fn new() -> Self {
        let counter = SharedKeyCounter::new();
        #[cfg(target_os = "macos")]
        {
            match macos_tap::try_start_tap(Arc::clone(&counter)) {
                Ok(guard) => Self {
                    counter,
                    listening: AtomicBool::new(true),
                    _tap_guard: Some(guard),
                },
                Err(reason) => {
                    tracing::warn!(
                        reason = %reason,
                        "input aggregate tap not started; collector will idle (counts=0)"
                    );
                    Self {
                        counter,
                        listening: AtomicBool::new(false),
                        _tap_guard: None,
                    }
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            Self {
                counter,
                listening: AtomicBool::new(false),
            }
        }
    }
}

impl InputCountProbe for SystemInputProbe {
    fn take_count(&self) -> CollectorResult<u64> {
        if !self.listening.load(Ordering::Relaxed) {
            return Ok(0);
        }
        Ok(self.counter.take())
    }

    fn accessibility_trusted(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            macos_tap::is_process_trusted()
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }
}

#[cfg(target_os = "macos")]
mod macos_tap {
    use std::ffi::c_void;
    use std::ptr;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread::{self, JoinHandle};

    use super::SharedKeyCounter;

    type CGEventRef = *mut c_void;
    type CFMachPortRef = *mut c_void;
    type CFRunLoopSourceRef = *mut c_void;
    type CFRunLoopRef = *mut c_void;
    type CGEventTapProxy = *mut c_void;
    type CGEventMask = u64;
    type CGEventType = u32;

    const K_CG_EVENT_KEY_DOWN: CGEventType = 10;
    const K_CG_HID_EVENT_TAP: u32 = 0;
    const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
    const K_CG_EVENT_TAP_OPTION_LISTEN_ONLY: u32 = 1;
    #[link(name = "ApplicationServices", kind = "framework")]
    unsafe extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGEventTapCreate(
            tap: u32,
            place: u32,
            options: u32,
            events_of_interest: CGEventMask,
            callback: extern "C" fn(
                CGEventTapProxy,
                CGEventType,
                CGEventRef,
                *mut c_void,
            ) -> CGEventRef,
            user_info: *mut c_void,
        ) -> CFMachPortRef;
        fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
        fn CGEventMaskBit(event_type: CGEventType) -> CGEventMask;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFMachPortCreateRunLoopSource(
            allocator: *const c_void,
            port: CFMachPortRef,
            order: i64,
        ) -> CFRunLoopSourceRef;
        fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: *const c_void);
        fn CFRunLoopGetCurrent() -> CFRunLoopRef;
        fn CFRunLoopRun();
        fn CFRunLoopStop(rl: CFRunLoopRef);
        fn CFRelease(cf: *const c_void);
        static kCFRunLoopCommonModes: *const c_void;
    }

    pub(super) fn is_process_trusted() -> bool {
        // SAFETY: Apple API, no pointers.
        unsafe { AXIsProcessTrusted() }
    }

    pub(super) struct TapGuard {
        stop: Arc<AtomicBool>,
        run_loop: Arc<std::sync::Mutex<Option<usize>>>,
        join: Option<JoinHandle<()>>,
    }

    impl std::fmt::Debug for TapGuard {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TapGuard")
                .field("listening", &true)
                .finish_non_exhaustive()
        }
    }

    impl Drop for TapGuard {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::SeqCst);
            if let Ok(guard) = self.run_loop.lock() {
                if let Some(rl) = *guard {
                    // SAFETY: pointer from CFRunLoopGetCurrent on the tap thread.
                    unsafe { CFRunLoopStop(rl as CFRunLoopRef) };
                }
            }
            if let Some(join) = self.join.take() {
                let _ = join.join();
            }
        }
    }

    extern "C" fn tap_callback(
        _proxy: CGEventTapProxy,
        event_type: CGEventType,
        event: CGEventRef,
        user_info: *mut c_void,
    ) -> CGEventRef {
        if event_type == K_CG_EVENT_KEY_DOWN && !user_info.is_null() {
            // SAFETY: user_info is Arc<SharedKeyCounter> raw pointer we own for tap lifetime.
            let counter = unsafe { &*(user_info as *const SharedKeyCounter) };
            counter.record_key_down();
        }
        event
    }

    pub(super) fn try_start_tap(counter: Arc<SharedKeyCounter>) -> Result<TapGuard, String> {
        if !is_process_trusted() {
            return Err("Accessibility not granted (AXIsProcessTrusted=false)".into());
        }

        let stop = Arc::new(AtomicBool::new(false));
        let run_loop = Arc::new(std::sync::Mutex::new(None));
        let stop_t = Arc::clone(&stop);
        let run_loop_t = Arc::clone(&run_loop);

        let join = thread::Builder::new()
            .name("biofocus-input-tap".into())
            .spawn(move || {
                // Leak a clone for the C callback; TapGuard drop stops the run loop first.
                let raw = Arc::into_raw(counter);
                // SAFETY: CoreGraphics / CoreFoundation FFI for listen-only key-down tap.
                unsafe {
                    let mask = CGEventMaskBit(K_CG_EVENT_KEY_DOWN);
                    let tap = CGEventTapCreate(
                        K_CG_HID_EVENT_TAP,
                        K_CG_HEAD_INSERT_EVENT_TAP,
                        K_CG_EVENT_TAP_OPTION_LISTEN_ONLY,
                        mask,
                        tap_callback,
                        raw as *mut c_void,
                    );
                    if tap.is_null() {
                        let _ = Arc::from_raw(raw);
                        tracing::warn!("CGEventTapCreate returned null");
                        return;
                    }
                    let source = CFMachPortCreateRunLoopSource(ptr::null(), tap, 0);
                    if source.is_null() {
                        CFRelease(tap as *const c_void);
                        let _ = Arc::from_raw(raw);
                        tracing::warn!("CFMachPortCreateRunLoopSource returned null");
                        return;
                    }
                    let rl = CFRunLoopGetCurrent();
                    if let Ok(mut g) = run_loop_t.lock() {
                        *g = Some(rl as usize);
                    }
                    CFRunLoopAddSource(rl, source, kCFRunLoopCommonModes);
                    CGEventTapEnable(tap, true);
                    while !stop_t.load(Ordering::SeqCst) {
                        CFRunLoopRun();
                        break;
                    }
                    CGEventTapEnable(tap, false);
                    CFRelease(source as *const c_void);
                    CFRelease(tap as *const c_void);
                    let _ = Arc::from_raw(raw);
                }
            })
            .map_err(|e| format!("failed to spawn input tap thread: {e}"))?;

        // Give the tap thread a moment; null tap is logged inside.
        thread::sleep(std::time::Duration::from_millis(20));

        Ok(TapGuard {
            stop,
            run_loop,
            join: Some(join),
        })
    }
}
