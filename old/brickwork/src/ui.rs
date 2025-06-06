use crate::widget::{Widget, layout::Direction};
use smallvec::SmallVec;
use vello::{
    Scene,
    kurbo::{Point, Rect, Vec2},
};

mod compute;
use compute::{WidgetLayout, WidgetRenderData};

pub struct UiContext {
    computed_layouts: Vec<WidgetLayout>,
    render_data: Vec<WidgetRenderData>,

    open_stack: Vec<OpenWidget>,

    pub(crate) scene: Scene,
}

#[derive(Debug, Clone)]
struct OpenWidget {
    index: usize,
    children: SmallVec<[usize; 1]>,
}

impl OpenWidget {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            children: SmallVec::new(),
        }
    }
    pub fn add_child(&mut self, child_index: usize) {
        self.children.push(child_index);
    }
}

impl UiContext {
    pub fn new() -> Self {
        Self {
            computed_layouts: Vec::new(),
            render_data: Vec::new(),

            open_stack: Vec::new(),

            scene: Scene::new(),
        }
    }

    pub fn set_ui(&mut self, ui: impl FnOnce(&mut Self)) {
        self.reset();
        ui(self);
        self.compute_layout();
    }

    fn compute_layout(&mut self) {
        if self.computed_layouts.is_empty() {
            return;
        }

        #[derive(Debug)]
        struct DfsBufferElement {
            index: usize,
            position: Point,
            child_offset: Vec2,
        }

        let root_padding = self.computed_layouts.first().unwrap().padding;
        let mut dfs_buffer = vec![DfsBufferElement {
            index: 0,
            position: Point::ORIGIN,
            child_offset: Vec2::new(root_padding.left.into(), root_padding.top.into()),
        }];
        let mut visited = vec![false; self.computed_layouts.len()];

        while dfs_buffer.len() > 0 {
            let element = dfs_buffer.last_mut().unwrap();
            let layout = &self.computed_layouts[element.index];
            let render_data = &mut self.render_data[element.index];

            if !visited[element.index] {
                visited[element.index] = true;

                let bounds = Rect::from_origin_size(element.position, layout.size.naive_size());

                let fill_color = render_data.class.get_color();

                use vello::{kurbo::Affine, peniko::Fill};
                dbg!(bounds);
                self.scene
                    .fill(Fill::NonZero, Affine::IDENTITY, fill_color, None, &bounds);
            } else {
                dfs_buffer.pop();
                continue;
            }

            let mut new_children = Vec::new(); //todo: avoid allocation somehow?
            new_children.reserve_exact(layout.children.len());

            for child_index in &layout.children {
                let child_layout = &self.computed_layouts[*child_index];
                let (child_width, child_height) = child_layout.size.naive_size();

                new_children.push(DfsBufferElement {
                    index: *child_index,
                    position: element.position + element.child_offset,
                    child_offset: Vec2::new(
                        child_layout.padding.left.into(),
                        child_layout.padding.top.into(),
                    ),
                });

                let child_gap: f64 = layout.child_gap.into();
                match layout.direction {
                    Direction::LeftToRight => element.child_offset.x += child_width + child_gap,
                    Direction::TopToBottom => element.child_offset.y += child_height + child_gap,
                }
            }

            dfs_buffer.extend(new_children.into_iter().rev());
        }
    }

    pub fn add(&mut self, widget: Widget) {
        self.add_parent(widget, |_| {});
    }

    pub fn add_parent(&mut self, widget: Widget, add_children: impl FnOnce(&mut Self)) {
        self.open_widget(widget);
        add_children(self);
        self.close_widget();
    }

    fn reset(&mut self) {
        self.scene.reset();
        self.computed_layouts.clear();
        self.open_stack.clear();
    }

    fn open_widget(&mut self, widget: Widget) {
        self.computed_layouts.push(WidgetLayout {
            direction: widget.direction,
            padding: widget.padding,
            child_gap: widget.child_gap,
            child_alignment: widget.child_alignment,
            size: widget.size,
            children: SmallVec::new(),
        });

        self.render_data.push(WidgetRenderData {
            class: widget.class,
        });

        let index = self.computed_layouts.len() - 1;
        if let Some(parent) = self.open_stack.last_mut() {
            parent.add_child(index);
        }

        self.open_stack.push(OpenWidget::new(index));
    }

    fn close_widget(&mut self) {
        let closed_widget = self.open_stack.pop().unwrap();
        let layout = self.computed_layouts.get_mut(closed_widget.index).unwrap();
        layout.children = closed_widget.children;
    }
}
