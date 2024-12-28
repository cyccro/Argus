use yrl::{WgpuState, YWindow};

mod reqx;

struct Yhandling {
    state: WgpuState<'static>,
}
impl yrl::YHandler for Yhandling {
    fn create(window: &yrl::winit::window::Window) -> Self {
        Self {
            state: WgpuState::new(window),
        }
    }
    fn handle_event(
        &mut self,
        ev_loop: &yrl::winit::event_loop::ActiveEventLoop,
        window_id: yrl::winit::window::WindowId,
        event: yrl::winit::event::WindowEvent,
    ) -> yrl::YMessage {
        yrl::YMessage::None
    }
}
#[tokio::main]
async fn main() {
    let mut window: YWindow<Yhandling> = yrl::YWindow::new(yrl::YWindowData {
        title: String::from("Hello Y"),
        width: 800,
        height: 600,
        icon: None,
        control_flow: yrl::winit::event_loop::ControlFlow::Poll,
    });
    window.run();
}
