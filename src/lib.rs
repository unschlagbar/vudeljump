
mod graphic;
mod game;

#[allow(non_snake_case, unused_variables)]
#[cfg(target_os = "android")]
mod android {
    use activity::AndroidApp;
    use iron_oxide::ui::{ErasedFnPointer, UiType};
    use winit::platform::android::EventLoopBuilderExtAndroid;
    use winit::platform::android::*;
    use log::info;
    use crate::game::{app::App, World};
    use winit::event_loop::{EventLoop, EventLoopBuilder};

    #[no_mangle]
    pub fn android_main(app: AndroidApp) {
        setup_panic_hook();
        android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::max()));
        log::info!("Running mainloop...");

        let event_loop: EventLoop<()> = EventLoopBuilder::default().with_android_app(app).build().unwrap();

        let mut application = App::run();

        {
            let mut ui = application.ui.borrow_mut();
            let element = unsafe { ui.get_element_mut(vec![2]).unwrap() };
    
            if let UiType::Button(button) = &mut element.inherit {
                button.on_press = ErasedFnPointer::from_associated_ui(&mut application.world, World::restart);
            }
        };
        
        info!("between");
        event_loop.run_app(&mut application).unwrap();
        info!("between3");
    }

    use std::panic;

    fn setup_panic_hook() {
        panic::set_hook(Box::new(|info| {
            log::error!("Panic occurred: {:?}", info);
        }));
    }
}