pub mod layout;
use layout::*;

pub mod state;
pub use state::{SubscribedValue, ValueProvider, WidgetValue};

#[derive(Debug, Clone, Default)]
pub struct Widget {
    // layout
    pub direction: Direction,
    pub padding: Padding,
    pub child_gap: u16,
    pub child_alignment: Alignment,
    pub size: Size,

    // visual
    pub class: WidgetClass,
    pub content: WidgetContent,

    pub clip: bool,

    // events
    pub on_hover: EventAction,
    pub on_click: EventAction,
}

#[derive(Debug, Clone, Default)]
pub enum WidgetContent {
    #[default]
    None,

    Text {
        content: WidgetValue<String>,
        role: TextRole,
    },
    // Image
}

#[derive(Debug, Clone, Copy, Default)]
pub enum WidgetClass {
    #[default]
    Surface,
    Primary,
    Secondary,
    Tertiary,
}

// TODO: get from stylesheet
use vello::peniko::Color;
impl WidgetClass {
    pub fn get_color(&self) -> Color {
        match self {
            WidgetClass::Surface => Color::from_rgb8(0xF0, 0xF0, 0xF0),
            WidgetClass::Primary => Color::from_rgb8(0x00, 0x80, 0xFF),
            WidgetClass::Secondary => Color::from_rgb8(0xFF, 0xA5, 0x00),
            WidgetClass::Tertiary => Color::from_rgb8(0x90, 0x90, 0x90),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum TextRole {
    #[default]
    Normal,

    Title,
    Subtitle,
    Caption,
}

use tokio::sync::mpsc::Sender;
#[derive(Debug, Clone, Default)]
pub enum EventAction {
    #[default]
    None,

    Message {
        id: u16,
        sender: Sender<Event>,
    },
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: u16,
    pub kind: EventKind,
}

#[derive(Debug, Clone, Copy)]
pub enum EventKind {
    Click,
    Hover,
}
