use android_activity::{AndroidApp, MainEvent, PollEvent};
use log::info;
use std::time::Duration;

mod render;

#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("SallyMC")
            .with_max_level(log::LevelFilter::Info),
    );
    info!("SallyMC starting (target: Android 10 / API 29, Vulkan 1.1)");

    let mut renderer: Option<render::Renderer> = None;
    let mut running = true;

    while running {
        app.poll_events(Some(Duration::from_millis(16)), |event| {
            if let PollEvent::Main(main_event) = event {
                match main_event {
                    MainEvent::InitWindow { .. } => {
                        if let Some(window) = app.native_window() {
                            renderer = render::Renderer::new(&window).ok();
                        }
                    }
                    MainEvent::TerminateWindow { .. } => {
                        renderer = None;
                    }
                    MainEvent::Destroy => running = false,
                    _ => {}
                }
            }
        });

        if let Some(r) = renderer.as_mut() {
            r.frame();
        }
    }

    info!("SallyMC exiting");
}
