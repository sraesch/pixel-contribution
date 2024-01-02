use std::{
    ffi::{CStr, CString},
    num::NonZeroU32,
};

use crate::{Error, EventHandler, Result};
use glutin::{
    config::{ConfigTemplateBuilder, GlConfig},
    context::{
        ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentGlContext,
        Version,
    },
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, SwapInterval},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use log::{debug, error, info};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use raw_window_handle::HasRawWindowHandle;

/// The options for creating the canvas.
pub struct CanvasOptions {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

pub fn create_and_run_canvas<H>(options: CanvasOptions, mut handler: H) -> Result<()>
where
    H: EventHandler,
{
    info!("Creating canvas...");

    // create event loop with control flow set to Poll, i.e., the event loop will run as fast as
    // possible
    debug!("Create event loop...");
    let event_loop = EventLoop::new().map_err(|e| Error::GraphicsAPI(format!("{}", e)))?;
    event_loop.set_control_flow(ControlFlow::Poll);

    debug!("Create windows builder...");
    let window_builder = if cfg!(wgl_backend) {
        WindowBuilder::new()
            .with_transparent(true)
            .with_title(options.title)
            .with_inner_size(LogicalSize::new(options.width, options.height))
    } else {
        WindowBuilder::new()
            .with_title(options.title)
            .with_inner_size(LogicalSize::new(options.width, options.height))
    };

    // The template will match only the configurations supporting rendering
    // to windows.
    //
    // We force transparency only on macOS, given that EGL on X11 doesn't
    // have it, but we still want to show window. The macOS situation is like
    // that, because we can query only one config at a time on it, but all
    // normal platforms will return multiple configs, so we can find the config
    // with transparency ourselves inside the `reduce`.
    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(cfg!(cgl_backend));

    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    debug!("Choose display configuration...");
    let (mut window, gl_config) = display_builder
        .build(&event_loop, template, |configs| {
            // Find the config with the maximum number of samples, so our triangle will
            // be smooth.
            configs
                .reduce(|accum, config| {
                    let transparency_check = config.supports_transparency().unwrap_or(false)
                        & !accum.supports_transparency().unwrap_or(false);

                    if transparency_check || config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        })
        .map_err(|e| Error::GraphicsAPI(format!("{}", e)))?;

    debug!("Display configuration: {:?}", gl_config);

    let raw_window_handle = window.as_ref().map(|window| window.raw_window_handle());

    // The display could be obtained from any object created by it, so we can
    // query it from the config.
    let gl_display = gl_config.display();

    info!("Load OpenGL function pointers...");
    gl::load_with(|symbol| {
        let symbol = CString::new(symbol).unwrap();
        let symbol = symbol.as_c_str();
        gl_display.get_proc_address(symbol) as *const _
    });

    // The context creation part. It can be created before surface and that's how
    // it's expected in multithreaded + multiwindow operation mode, since you
    // can send NotCurrentContext, but not Surface.
    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 0))))
        .build(raw_window_handle);

    let mut not_current_gl_context = Some(unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .expect("failed to create context")
    });

    let mut state = None;
    let mut is_initialized = false;
    event_loop
        .run(move |event, window_target| {
            match event {
                Event::Resumed => {
                    #[cfg(android_platform)]
                    info!("Android window available");

                    let window = window.take().unwrap_or_else(|| {
                        let window_builder = WindowBuilder::new().with_transparent(true);
                        glutin_winit::finalize_window(window_target, window_builder, &gl_config)
                            .unwrap()
                    });

                    let attrs = window.build_surface_attributes(Default::default());
                    let gl_surface = unsafe {
                        gl_config
                            .display()
                            .create_window_surface(&gl_config, &attrs)
                            .unwrap()
                    };

                    // Make it current.
                    let gl_context = not_current_gl_context
                        .take()
                        .unwrap()
                        .make_current(&gl_surface)
                        .unwrap();

                    // The context needs to be current for the Renderer to set up shaders and
                    // buffers. It also performs function loading, which needs a current context on
                    // WGL.
                    if !is_initialized {
                        let s = unsafe { CStr::from_ptr(gl::GetString(gl::VENDOR) as *const i8) }
                            .to_str()
                            .expect("Failed to get vendor string");
                        info!("Vendor: {}", s);
                        let s = unsafe { CStr::from_ptr(gl::GetString(gl::RENDERER) as *const i8) }
                            .to_str()
                            .expect("Failed to get renderer string");
                        info!("Renderer: {}", s);
                        let s = unsafe { CStr::from_ptr(gl::GetString(gl::VERSION) as *const i8) }
                            .to_str()
                            .expect("Failed to get version string");
                        info!("Version: {}", s);

                        if let Err(err) = handler.setup() {
                            error!("Error during setup: {}", err);
                        }

                        is_initialized = true;
                    }

                    // Try setting vsync.
                    if let Err(res) = gl_surface.set_swap_interval(
                        &gl_context,
                        SwapInterval::Wait(NonZeroU32::new(1).unwrap()),
                    ) {
                        error!("Error setting vsync: {res:?}");
                    }

                    assert!(state.replace((gl_context, gl_surface, window)).is_none());
                }
                Event::Suspended => {
                    // This event is only raised on Android, where the backing NativeWindow for a GL
                    // Surface can appear and disappear at any moment.
                    info!("Android window removed");

                    // Destroy the GL Surface and un-current the GL Context before ndk-glue releases
                    // the window back to the system.
                    let (gl_context, ..) = state.take().unwrap();
                    assert!(not_current_gl_context
                        .replace(gl_context.make_not_current().unwrap())
                        .is_none());
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        if size.width != 0 && size.height != 0 {
                            // Some platforms like EGL require resizing GL surface to update the size
                            // Notable platforms here are Wayland and macOS, other don't require it
                            // and the function is no-op, but it's wise to resize it for portability
                            // reasons.
                            if let Some((gl_context, gl_surface, _)) = &state {
                                gl_surface.resize(
                                    gl_context,
                                    NonZeroU32::new(size.width).unwrap(),
                                    NonZeroU32::new(size.height).unwrap(),
                                );
                                handler.resize(size.width, size.height);
                            }
                        }
                    }
                    WindowEvent::CloseRequested => window_target.exit(),
                    _ => (),
                },
                Event::AboutToWait => {
                    if let Some((gl_context, gl_surface, window)) = &state {
                        handler.next_frame();
                        window.request_redraw();

                        gl_surface.swap_buffers(gl_context).unwrap();
                    }
                }
                _ => (),
            }
        })
        .map_err(|e| Error::GraphicsAPI(format!("{}", e)))?;

    Ok(())
}

// pub fn create_and_run_canvas<H>(options: CanvasOptions, mut handler: H) -> Result<()>
// where
//     H: EventHandler,
// {
//     info!("Creating canvas...");

//     let event_loop = EventLoop::new().map_err(|e| Error::GraphicsAPI(format!("{}", e)))?;
//     // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
//     // dispatched any events. This is ideal for games and similar applications.
//     event_loop.set_control_flow(ControlFlow::Poll);

//     debug!("Create window builder...");
//     let window_builder = WindowBuilder::new()
//         .with_title(options.title)
//         .with_inner_size(LogicalSize::new(options.width, options.height));

//     // The template will match only the configurations supporting rendering
//     // to windows.
//     let template = ConfigTemplateBuilder::new().with_alpha_size(8);

//     let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

//     let (mut window, gl_config) = display_builder
//         .build(&event_loop, template, |configs| {
//             // Find the config with the maximum number of samples, so our triangle will
//             // be smooth.
//             configs
//                 .reduce(|accum, config| {
//                     let transparency_check = config.supports_transparency().unwrap_or(false)
//                         & !accum.supports_transparency().unwrap_or(false);

//                     if transparency_check || config.num_samples() > accum.num_samples() {
//                         config
//                     } else {
//                         accum
//                     }
//                 })
//                 .unwrap()
//         })
//         .map_err(|e| Error::GraphicsAPI(format!("{}", e)))?;

//     info!(
//         "Choose the following display configuration: {:?}",
//         gl_config
//     );

//     // call setup function of the event handler
//     if let Err(e) = handler.setup() {
//         error!("Error during setup: {}", e);
//         return Err(Error::Internal(
//             "Stopped due to error in setup:".to_string(),
//         ));
//     }

//     event_loop
//         .run(move |event, elwt| {
//             match event {
//                 Event::WindowEvent {
//                     event: WindowEvent::CloseRequested,
//                     ..
//                 } => {
//                     info!("The close button was pressed; stopping");
//                     handler.stop();
//                     elwt.exit();
//                 }
//                 Event::AboutToWait => {
//                     // Application update code.

//                     // Queue a RedrawRequested event.
//                     //
//                     // You only need to call this if you've determined that you need to redraw in
//                     // applications which do not always need to. Applications that redraw continuously
//                     // can render here instead.
//                     window.request_redraw();
//                 }
//                 Event::WindowEvent {
//                     event: WindowEvent::RedrawRequested,
//                     ..
//                 } => {
//                     // Redraw the application.
//                     //
//                     // It's preferable for applications that do not render continuously to render in
//                     // this event rather than in AboutToWait, since rendering in here allows
//                     // the program to gracefully handle redraws requested by the OS.
//                 }
//                 _ => (),
//             }
//         })
//         .map_err(|e| Error::GraphicsAPI(format!("{}", e)))?;

//     Ok(())
// }
