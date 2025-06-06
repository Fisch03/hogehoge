use brickwork::{
    Context, Surface, Widget,
    widget::{
        WidgetClass,
        layout::{Padding, Size},
    },
};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

#[derive(Debug)]
enum RenderState<'s> {
    Active {
        surface: Box<Surface<'s>>,
        window: Arc<Window>,
    },
    Suspended {
        window: Option<Arc<Window>>,
    },
}

struct App<'s> {
    context: Context,

    state: RenderState<'s>,
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let RenderState::Suspended { window } = &mut self.state else {
            return;
        };

        let window = window.take().unwrap_or_else(|| {
            let window = event_loop
                .create_window(Window::default_attributes().with_title("2hoge"))
                .unwrap();

            Arc::new(window)
        });

        let size: (u32, u32) = window.inner_size().into();
        let surface = pollster::block_on(self.context.create_surface(window.clone(), size))
            .expect("Failed to create surface");

        self.state = RenderState::Active {
            surface: Box::new(surface),
            window,
        };
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if let RenderState::Active { window, .. } = &self.state {
            self.state = RenderState::Suspended {
                window: Some(window.clone()),
            };
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let surface = match &mut self.state {
            RenderState::Active { surface, window } if window.id() == window_id => surface,
            _ => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => self.context.resize_surface(surface, size.into()),

            WindowEvent::RedrawRequested => {
                self.context.render(surface);
            }

            _ => {}
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut app = App {
        context: Context::new(),
        state: RenderState::Suspended { window: None },
    };

    app.context.set_ui(|ctx| {
        ctx.add_parent(
            Widget {
                size: Size::fixed(960, 540),
                padding: Padding::all(30),
                child_gap: 10,
                ..Default::default()
            },
            |ctx| {
                ctx.add(Widget {
                    size: Size::fixed(300, 300),
                    class: WidgetClass::Primary,
                    ..Default::default()
                });
                ctx.add(Widget {
                    size: Size::fixed(300, 300),
                    class: WidgetClass::Secondary,
                    ..Default::default()
                });

                // ctx.add_parent(
                //     Widget {
                //         class: WidgetClass::Primary,
                //         ..Default::default()
                //     },
                //     |ctx| {
                //         ctx.add(Widget {
                //             class: WidgetClass::Secondary,
                //             ..Default::default()
                //         });
                //         ctx.add(Widget {
                //             class: WidgetClass::Secondary,
                //             ..Default::default()
                //         });
                //     },
                // );
            },
        )
    });

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run_app(&mut app)
        .expect("Failed to run the event loop");

    Ok(())
}
