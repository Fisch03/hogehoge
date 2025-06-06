use crate::widget::{WidgetClass, layout::*};
use smallvec::SmallVec;

#[derive(Debug, Clone)]
pub struct WidgetLayout {
    pub direction: Direction,
    pub padding: Padding,
    pub child_gap: u16,
    pub child_alignment: Alignment,
    pub size: Size,

    pub children: SmallVec<[usize; 1]>,
}

pub struct WidgetRenderData {
    pub class: WidgetClass,
}
