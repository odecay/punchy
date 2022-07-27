use js_sys::Promise;
use networking::client::connection::Connection;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use winit::event_loop::EventLoop;

#[wasm_bindgen]
pub struct Game {}

#[derive(Serialize, Debug, Copy, Clone)]
pub struct Test(u32);

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(
        &self,
        assets_location: String,
        session_id: String,
        asset_url: String,
        access_token: String,
        udp_url: String,
        tcp_url: String,
    ) -> Promise {
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();

        wasm_logger::init(wasm_logger::Config::default());
        future_to_promise(async move {
            // ~130MB

            let mut connection = Connection::new("session").unwrap();
            let connection_receiver = connection
                .connect(
                    tcp_url.clone(),
                    udp_url.clone(),
                    session_id.clone(),
                    access_token.clone(),
                )
                .await;

            let test = Test(3);
            let event_loop = EventLoop::new();
            event_loop.run(move |event, _, control_flow| {
                connection.send_unreliable_with(test);
                log::info!("Looping... this will not log yet");
            });
        })
    }
}
