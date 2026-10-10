//! Dashboard window frame.
//!
//! Hide-on-close keeps the `dashboard` webview alive. On macOS, closing or
//! miniaturizing that window while it is fullscreen (or zoomed) drops the
//! AppKit frame to a tiny size, and the next show reuses it. Remember the last
//! real windowed size and put it back before the window is hidden.

use std::sync::{Mutex, MutexGuard};

use tauri::{AppHandle, LogicalSize, Manager, PhysicalSize, WebviewWindow, WindowEvent};

/// Logical size from `tauri.conf.json` (`dashboard` window).
pub const DEFAULT_WIDTH: f64 = 720.0;
pub const DEFAULT_HEIGHT: f64 = 520.0;
/// `minWidth` / `minHeight` from the same window config.
pub const MIN_WIDTH: f64 = 480.0;
pub const MIN_HEIGHT: f64 = 360.0;

const SIZE_SLOP: f64 = 8.0;
const MAX_RESTORE_ATTEMPTS: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameSize {
    pub width: f64,
    pub height: f64,
}

impl FrameSize {
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClosePlan {
    /// Already windowed: hide immediately.
    Hide,
    /// Leave fullscreen / zoom, then hide once the windowed frame is back.
    LeaveExpanded(FrameSize),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResizePlan {
    None,
    Restore(FrameSize),
    Hide,
}

#[derive(Clone, Debug)]
pub struct DashboardFrame {
    last_good: FrameSize,
    was_expanded: bool,
    hide_after_restore: bool,
    restore_attempts: u8,
}

impl DashboardFrame {
    pub fn new() -> Self {
        Self {
            last_good: FrameSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT),
            was_expanded: false,
            hide_after_restore: false,
            restore_attempts: 0,
        }
    }

    pub fn last_good(&self) -> FrameSize {
        self.last_good
    }

    pub fn plan_close(&mut self, expanded: bool) -> ClosePlan {
        if expanded {
            self.was_expanded = true;
            self.hide_after_restore = true;
            ClosePlan::LeaveExpanded(self.last_good)
        } else {
            self.hide_after_restore = false;
            ClosePlan::Hide
        }
    }

    /// `expanded` is native fullscreen or zoom. Sizes below the configured
    /// minimum are a collapsed frame, not a user resize.
    pub fn on_resized(&mut self, expanded: bool, size: FrameSize) -> ResizePlan {
        if expanded {
            self.was_expanded = true;
            self.restore_attempts = 0;
            return ResizePlan::None;
        }

        if self.was_expanded {
            self.was_expanded = false;
            self.restore_attempts = 0;
            if !near(size, self.last_good) {
                self.restore_attempts = 1;
                return ResizePlan::Restore(self.last_good);
            }
        } else if !is_usable(size) {
            if self.restore_attempts >= MAX_RESTORE_ATTEMPTS {
                return self.take_hide();
            }
            self.restore_attempts += 1;
            return ResizePlan::Restore(self.last_good);
        } else {
            self.restore_attempts = 0;
            self.last_good = size;
        }

        self.take_hide()
    }

    /// Size to apply when showing a hidden or collapsed dashboard.
    /// A visible fullscreen window is left alone so menubar reopen only focuses it.
    pub fn repair_on_show(
        &self,
        expanded: bool,
        visible: bool,
        size: Option<FrameSize>,
    ) -> Option<FrameSize> {
        if expanded && visible {
            return None;
        }
        if expanded && !visible {
            return Some(self.last_good);
        }
        match size {
            Some(size) if !is_usable(size) => Some(self.last_good),
            _ => None,
        }
    }

    fn take_hide(&mut self) -> ResizePlan {
        if self.hide_after_restore {
            self.hide_after_restore = false;
            ResizePlan::Hide
        } else {
            ResizePlan::None
        }
    }
}

impl Default for DashboardFrame {
    fn default() -> Self {
        Self::new()
    }
}

fn is_usable(size: FrameSize) -> bool {
    size.width + 0.5 >= MIN_WIDTH && size.height + 0.5 >= MIN_HEIGHT
}

fn near(a: FrameSize, b: FrameSize) -> bool {
    (a.width - b.width).abs() <= SIZE_SLOP && (a.height - b.height).abs() <= SIZE_SLOP
}

fn lock(frame: &Mutex<DashboardFrame>) -> MutexGuard<'_, DashboardFrame> {
    match frame.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Hooks hide-on-close and frame recovery for the preconfigured dashboard window.
pub fn install(app: &AppHandle) {
    app.manage(Mutex::new(DashboardFrame::new()));
    let Some(window) = app.get_webview_window("dashboard") else {
        return;
    };
    let handle = app.clone();
    let events = window.clone();
    window.on_window_event(move |event| handle_event(&handle, &events, event));
}

/// Shows the dashboard. Repairs a frame collapsed by a fullscreen close or minimize.
pub fn show(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("dashboard") else {
        return Err("Dashboard window is not available.".into());
    };

    let visible = window.is_visible().unwrap_or(false);
    let fullscreen = window.is_fullscreen().unwrap_or(false);
    let maximized = window.is_maximized().unwrap_or(false);
    let size = logical_inner(&window);
    let restore = app
        .try_state::<Mutex<DashboardFrame>>()
        .and_then(|frame| lock(&frame).repair_on_show(fullscreen || maximized, visible, size));

    if let Some(size) = restore {
        leave_expanded(&window);
        apply_size(&window, size);
    }
    window.unminimize().map_err(|err| err.to_string())?;
    window.show().map_err(|err| err.to_string())?;
    window.set_focus().map_err(|err| err.to_string())?;
    Ok(())
}

fn handle_event(app: &AppHandle, window: &WebviewWindow, event: &WindowEvent) {
    let Some(frame) = app.try_state::<Mutex<DashboardFrame>>() else {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = window.hide();
        }
        return;
    };

    match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            let expanded = is_expanded(window);
            let plan = lock(&frame).plan_close(expanded);
            match plan {
                ClosePlan::Hide => {
                    let _ = window.hide();
                }
                ClosePlan::LeaveExpanded(size) => {
                    leave_expanded(window);
                    apply_size(window, size);
                }
            }
        }
        WindowEvent::Resized(physical) => {
            let size = to_logical(window, *physical);
            let expanded = is_expanded(window);
            let plan = lock(&frame).on_resized(expanded, size);
            match plan {
                ResizePlan::None => {}
                ResizePlan::Restore(size) => apply_size(window, size),
                ResizePlan::Hide => {
                    let _ = window.hide();
                }
            }
        }
        _ => {}
    }
}

fn is_expanded(window: &WebviewWindow) -> bool {
    window.is_fullscreen().unwrap_or(false) || window.is_maximized().unwrap_or(false)
}

fn leave_expanded(window: &WebviewWindow) {
    if window.is_fullscreen().unwrap_or(false) {
        let _ = window.set_fullscreen(false);
    }
    if window.is_maximized().unwrap_or(false) {
        let _ = window.unmaximize();
    }
}

fn apply_size(window: &WebviewWindow, size: FrameSize) {
    let _ = window.set_size(LogicalSize::new(size.width, size.height));
}

fn logical_inner(window: &WebviewWindow) -> Option<FrameSize> {
    let physical = window.inner_size().ok()?;
    Some(to_logical(window, physical))
}

fn to_logical(window: &WebviewWindow, physical: PhysicalSize<u32>) -> FrameSize {
    let scale = window.scale_factor().unwrap_or(1.0);
    let scale = if scale.is_finite() && scale > f64::EPSILON {
        scale
    } else {
        1.0
    };
    FrameSize::new(
        physical.width as f64 / scale,
        physical.height as f64 / scale,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame_after_user_resize(width: f64, height: f64) -> DashboardFrame {
        let mut frame = DashboardFrame::new();
        assert_eq!(
            frame.on_resized(false, FrameSize::new(width, height)),
            ResizePlan::None
        );
        frame
    }

    #[test]
    fn user_resize_is_remembered() {
        let frame = frame_after_user_resize(900.0, 640.0);
        assert_eq!(frame.last_good(), FrameSize::new(900.0, 640.0));
    }

    #[test]
    fn fullscreen_resize_does_not_replace_windowed_size() {
        let mut frame = frame_after_user_resize(900.0, 640.0);
        assert_eq!(
            frame.on_resized(true, FrameSize::new(1728.0, 1117.0)),
            ResizePlan::None
        );
        assert_eq!(frame.last_good(), FrameSize::new(900.0, 640.0));
    }

    #[test]
    fn leaving_fullscreen_into_a_tiny_frame_restores_last_size() {
        let mut frame = frame_after_user_resize(900.0, 640.0);
        let _ = frame.on_resized(true, FrameSize::new(1728.0, 1117.0));
        assert_eq!(
            frame.on_resized(false, FrameSize::new(120.0, 80.0)),
            ResizePlan::Restore(FrameSize::new(900.0, 640.0))
        );
        assert_eq!(frame.last_good(), FrameSize::new(900.0, 640.0));
    }

    #[test]
    fn leaving_fullscreen_into_the_minimum_still_restores_the_larger_frame() {
        let mut frame = frame_after_user_resize(900.0, 640.0);
        let _ = frame.on_resized(true, FrameSize::new(1728.0, 1117.0));
        assert_eq!(
            frame.on_resized(false, FrameSize::new(MIN_WIDTH, MIN_HEIGHT)),
            ResizePlan::Restore(FrameSize::new(900.0, 640.0))
        );
    }

    #[test]
    fn close_while_fullscreen_hides_only_after_the_windowed_frame_returns() {
        let mut frame = frame_after_user_resize(900.0, 640.0);
        let _ = frame.on_resized(true, FrameSize::new(1728.0, 1117.0));
        assert_eq!(
            frame.plan_close(true),
            ClosePlan::LeaveExpanded(FrameSize::new(900.0, 640.0))
        );
        assert_eq!(
            frame.on_resized(false, FrameSize::new(120.0, 80.0)),
            ResizePlan::Restore(FrameSize::new(900.0, 640.0))
        );
        assert_eq!(
            frame.on_resized(false, FrameSize::new(900.0, 640.0)),
            ResizePlan::Hide
        );
    }

    #[test]
    fn close_while_windowed_hides_immediately() {
        let mut frame = frame_after_user_resize(800.0, 600.0);
        assert_eq!(frame.plan_close(false), ClosePlan::Hide);
    }

    #[test]
    fn green_button_exit_at_the_same_size_does_not_hide() {
        let mut frame = frame_after_user_resize(800.0, 600.0);
        let _ = frame.on_resized(true, FrameSize::new(1728.0, 1117.0));
        assert_eq!(
            frame.on_resized(false, FrameSize::new(802.0, 598.0)),
            ResizePlan::None
        );
        assert_eq!(frame.last_good().width, 800.0);
    }

    #[test]
    fn collapsed_frame_does_not_loop_forever() {
        let mut frame = DashboardFrame::new();
        for _ in 0..MAX_RESTORE_ATTEMPTS {
            assert_eq!(
                frame.on_resized(false, FrameSize::new(10.0, 10.0)),
                ResizePlan::Restore(FrameSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
            );
        }
        assert_eq!(
            frame.on_resized(false, FrameSize::new(10.0, 10.0)),
            ResizePlan::None
        );
    }

    #[test]
    fn show_repairs_a_hidden_fullscreen_window_and_a_visible_tiny_one() {
        let frame = frame_after_user_resize(900.0, 640.0);
        let good = FrameSize::new(900.0, 640.0);
        assert_eq!(
            frame.repair_on_show(true, false, Some(FrameSize::new(120.0, 80.0))),
            Some(good)
        );
        assert_eq!(
            frame.repair_on_show(false, true, Some(FrameSize::new(120.0, 80.0))),
            Some(good)
        );
        assert_eq!(
            frame.repair_on_show(true, true, Some(FrameSize::new(1728.0, 1117.0))),
            None
        );
        assert_eq!(frame.repair_on_show(false, true, Some(good)), None);
    }
}
