use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    window::{Icon, Window, WindowAttributes, WindowId},
};

pub enum YMessage {
    Block(Vec<YMessage>),
    RequestWindow,
    None,
}

pub trait YHandler {
    fn handle_event(
        &mut self,
        ev_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) -> YMessage;
    fn window_req(&mut self, _window: &Window) {}
    fn create(window: &Window) -> Self;
}

pub struct YWindowData {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub icon: Option<Icon>,
    pub control_flow: ControlFlow,
}

pub struct YWindow<T> {
    data: YWindowData,
    handler: Option<T>,
    window: Option<Window>,
}
impl<T> YWindow<T>
where
    T: YHandler,
{
    pub fn new(data: YWindowData) -> Self {
        Self {
            data,
            window: None,
            handler: None,
        }
    }
    pub fn handler(&self) -> Option<&T> {
        self.handler.as_ref()
    }
    pub fn handler_mut(&mut self) -> Option<&mut T> {
        if let Some(handler) = &mut self.handler {
            Some(handler)
        } else {
            None
        }
    }
    pub fn run(&mut self) {
        let evloop = winit::event_loop::EventLoop::new().unwrap();
        evloop.set_control_flow(self.data.control_flow);
        evloop.run_app(self).unwrap()
    }
    fn handle_single_message(&mut self, message: YMessage) {
        if matches!(message, YMessage::Block(_)) {
            self.handle_message(message);
            return;
        }
        let Some(handler) = &mut self.handler else {
            return;
        };
        match message {
            YMessage::RequestWindow => handler.window_req(self.window.as_ref().unwrap()),
            YMessage::None => {}
            _ => {}
        }
    }
    fn handle_message(&mut self, message: YMessage) {
        match message {
            YMessage::Block(msgs) => {
                for msg in msgs {
                    self.handle_single_message(msg);
                }
            }
            msg => self.handle_single_message(msg),
        }
    }
}
impl<T> ApplicationHandler for YWindow<T>
where
    T: YHandler,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_inner_size(PhysicalSize::new(self.data.width, self.data.height))
                    .with_resizable(true)
                    .with_window_icon(self.data.icon.clone())
                    .with_title(&self.data.title),
            )
            .unwrap();

        self.handler = Some(T::create(&window));
        self.window = Some(window);
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::Destroyed => {
                return;
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
                self.window = None;
                self.handler = None;
                return;
            }
            _ => {}
        }
        let msg = self
            .handler_mut()
            .unwrap()
            .handle_event(event_loop, window_id, event);
        self.handle_message(msg);
    }
}
