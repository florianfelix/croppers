#![allow(
    clippy::new_without_default,
    clippy::collapsible_match,
    clippy::collapsible_if,
    clippy::single_match,
    clippy::derivable_impls,
    clippy::field_reassign_with_default
)]
// #![allow(unused)]
use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use clap::Parser;
use tracing::warn;
use tracing::{info, trace};
#[cfg(target_os = "linux")]
use winit::platform::{wayland::ActiveEventLoopExtWayland, x11::ActiveEventLoopExtX11};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    keyboard::{Key, ModifiersState, NamedKey},
    platform::{
        modifier_supplement::KeyEventExtModifierSupplement, wayland::WindowAttributesExtWayland,
    },
    window::Window,
};

use crate::{
    app_state::AppState, compute::ComputeResult, render_state::RenderState, utils::CroppersConfig,
};

pub mod app_state;
pub mod compute;
pub mod render_state;
pub mod utils;

#[derive(Debug, Clone)]
pub enum Message {
    Empty,
    ComputeResult(ComputeResult),
    FileSaved(PathBuf),
    FileSaveError(String),
    FileOpened(PathBuf),
}

pub struct Croppers {
    renderer: Option<RenderState>,
    app_state: AppState,
    proxy: Option<EventLoopProxy<Message>>,
    modifiers: ModifiersState,
}

impl Croppers {
    pub fn new() -> Self {
        utils::init_logging();
        let cli = utils::Cli::parse();

        let config = CroppersConfig::load(&cli).unwrap_or_default();

        info!("{:#?}", cli);
        Self {
            renderer: None,
            app_state: AppState::new(cli, config),
            proxy: None,
            modifiers: ModifiersState::default(),
        }
    }
    pub fn run(&mut self) -> Result<()> {
        let event_loop = EventLoop::<Message>::with_user_event().build()?;
        self.proxy = Some(event_loop.create_proxy());
        self.app_state.proxy = Some(event_loop.create_proxy());
        event_loop.run_app(self)?;
        Ok(())
    }
}

impl ApplicationHandler<Message> for Croppers {
    #[tracing::instrument(skip_all)]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(target_os = "linux")]
        if event_loop.is_x11() {
            warn!("Winit is running on X11 (XWayland).");
        } else if event_loop.is_wayland() {
            warn!("Winit is running on Wayland.");
        }
        if self.renderer.is_some() {
            return;
        }
        let Some(proxy) = &self.proxy else {
            return;
        };

        let fullscreen = if self.app_state.config.fullscreen {
            // If primary monitor is None, gracefully fall back to the first monitor found
            let monitor = event_loop
                .primary_monitor()
                .or_else(|| event_loop.available_monitors().next());

            monitor.map(|m| winit::window::Fullscreen::Borderless(Some(m)))
        } else {
            None
        };

        let window_attributes = Window::default_attributes()
            .with_title("Croppers")
            .with_fullscreen(fullscreen)
            .with_name(&self.app_state.config.app_id, &self.app_state.config.app_id)
            .with_window_icon(load_window_icon())
            .with_visible(false);
        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create winit window."),
        );
        let window_clone = window.clone();

        let renderer = pollster::block_on(RenderState::new(window_clone, &mut self.app_state));

        let device = renderer.device.clone();
        let queue = renderer.queue.clone();
        let rx_job = self.app_state.rx_job.clone();

        compute::spawn_worker(device, queue, proxy.clone(), rx_job);

        let device = renderer.device.clone();
        let queue = renderer.queue.clone();
        self.app_state.gpu = Some((device, queue));

        self.app_state.set_srcimg_texture_from_source();

        renderer.window.set_visible(true);
        self.renderer = Some(renderer);
    }
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.renderer = None;
    }
    #[tracing::instrument(skip_all)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: Message) {
        match event {
            Message::Empty => trace!("Received empty proxy message. Probably for redrawing"),
            Message::ComputeResult(compute_result) => {
                info!("Received Compute Result");
                self.app_state.handle_compute_result(compute_result);
            }
            Message::FileSaved(path) => {
                info!("Saved file: {:?}", path);
                self.app_state.action_on_save(path);
            }
            Message::FileSaveError(e) => {
                self.app_state
                    .ui_state
                    .notifications
                    .push(&format!("Failed to save: {}", e), true);
            }
            Message::FileOpened(path) => {
                info!("Opening file from dialog: {:?}", path);
                self.app_state.try_open_file(&path);
            }
        }
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.window.request_redraw();
        };
    }

    #[tracing::instrument(skip_all)]
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        window_event: winit::event::WindowEvent,
    ) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        let is_input_event = matches!(
            window_event,
            WindowEvent::CursorMoved { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::KeyboardInput { .. }
                | WindowEvent::Touch { .. }
        );

        let response = renderer
            .ui_renderer
            .egui_winit
            .on_window_event(&renderer.window, &window_event);

        if response.repaint && is_input_event {
            renderer.window.request_redraw();
        }
        if self.app_state.effect_stack.needs_rebuild
            || self.app_state.source_dirty
            || self.app_state.needs_compute
        {
            renderer.window.request_redraw();
        }

        match window_event {
            WindowEvent::CloseRequested => {
                self.renderer = None;
                event_loop.exit();
            }
            WindowEvent::HoveredFile(path) => {
                info!("Hovered file: {:?}", path);
                renderer.window.request_redraw();
                // self.app_state.hovered_file = Some(path)
            }
            WindowEvent::DroppedFile(path) => {
                info!("Dropped file: {:?}", path);
                renderer.window.request_redraw();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::Resized(physical_size) => renderer.handle_resize(physical_size),
            WindowEvent::RedrawRequested => {
                if self.app_state.update_before_reddraw() {
                    renderer.window.request_redraw();
                };

                renderer.update(&mut self.app_state);

                match renderer.render(&mut self.app_state) {
                    Ok(_) => {}
                    Err(wgpu::hal::SurfaceError::Lost) => {
                        let _ = renderer.device.poll(wgpu::PollType::Wait {
                            submission_index: None,
                            timeout: None, // Wait indefinitely until idle
                        });
                        renderer
                            .surface
                            .configure(&renderer.device, &renderer.config)
                    }
                    Err(wgpu::hal::SurfaceError::Device(wgpu::hal::DeviceError::OutOfMemory)) => {
                        event_loop.exit()
                    }
                    Err(e) => eprintln!("Render error: {e:?}"),
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if !response.consumed {
                    self.handle_keyboard_event(event, event_loop);
                }
            }
            _ => {}
        }
    }
}

impl Croppers {
    pub fn handle_keyboard_event(&mut self, event: KeyEvent, event_loop: &ActiveEventLoop) {
        if event.state == ElementState::Pressed && !event.repeat {
            if event.logical_key.as_ref() == Key::Character("?") {
                self.app_state.ui_state.toggle_settings();
                return;
            }
            match event.key_without_modifiers().as_ref() {
                Key::Character("w") => self.app_state.ui_state.toggle_panel(),
                Key::Character("?") => self.app_state.ui_state.toggle_settings(),
                Key::Character("q") => {
                    self.renderer = None; // Clean up before exit
                    event_loop.exit();
                }
                Key::Character("n") | Key::Named(NamedKey::ArrowRight) => {
                    self.app_state.open_next()
                }
                Key::Character("b") | Key::Named(NamedKey::ArrowLeft) => {
                    self.app_state.open_previous()
                }
                Key::Character("c") => {
                    self.app_state.crop_toggle();
                }
                Key::Character("e") => {
                    self.app_state.effects_toggle();
                }
                Key::Character("r") => {
                    self.app_state.ui_state.reset_view();
                }
                Key::Named(NamedKey::Enter) => {
                    self.app_state.send_crop_job();
                }
                Key::Character("o") => {
                    if self.modifiers.control_key() {
                        self.app_state.open_file_dialog();
                    }
                }
                Key::Character("s") => {
                    let ctrl = self.modifiers.control_key();
                    let increment = self.modifiers.shift_key();
                    if ctrl {
                        self.app_state.save_keyboard_shortcut(increment);
                    }
                }
                Key::Named(NamedKey::Delete) => {
                    self.app_state.delete_current_file();
                }
                _ => {}
            }
        }
    }
}

/// Loads the embedded application icon for the winit window.
/// Note: Wayland compositors ignore this and use the icon from the .desktop file (matched via app_id).
fn load_window_icon() -> Option<winit::window::Icon> {
    const ICON_BYTES: &[u8] = include_bytes!("../assets/croppers.png");
    let img = match image::load_from_memory(ICON_BYTES) {
        Ok(img) => img.into_rgba8(),
        Err(e) => {
            warn!("Failed to decode window icon: {e}");
            return None;
        }
    };
    let (w, h) = img.dimensions();
    winit::window::Icon::from_rgba(img.into_raw(), w, h)
        .map_err(|e| warn!("Failed to create window icon: {e}"))
        .ok()
}
