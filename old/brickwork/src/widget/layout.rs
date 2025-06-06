#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum Direction {
    #[default]
    LeftToRight,
    TopToBottom,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Padding {
    pub left: u16,
    pub right: u16,
    pub top: u16,
    pub bottom: u16,
}

impl Padding {
    pub const fn all(value: u16) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ChildAlignment {
    pub horizontal: Alignment,
    pub vertical: Alignment,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum Alignment {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Size {
    pub width: SizeAxis,
    pub height: SizeAxis,
}

#[derive(Debug, Clone, Copy)]
pub struct SizeAxis {
    pub min: u16,
    pub max: u16,
    pub kind: SizeKind,
}

#[derive(Debug, Clone, Copy)]
pub enum SizeKind {
    Fit,
    Grow,
}

impl Size {
    pub const FILL: Self = Self {
        width: SizeAxis::FILL,
        height: SizeAxis::FILL,
    };
    pub const FIT: Self = Self {
        width: SizeAxis::FIT,
        height: SizeAxis::FIT,
    };

    pub fn fixed(width: u16, height: u16) -> Self {
        Self {
            width: SizeAxis::fixed(width),
            height: SizeAxis::fixed(height),
        }
    }

    pub fn naive_size(&self) -> (f64, f64) {
        // todo: replace with actual calculated sizing

        (self.width.min as f64, self.height.min as f64)
    }
}

impl SizeAxis {
    pub const FILL: Self = Self {
        min: 0,
        max: u16::MAX,
        kind: SizeKind::Grow,
    };

    pub const FIT: Self = Self {
        min: 0,
        max: u16::MAX,
        kind: SizeKind::Fit,
    };

    pub fn fixed(value: u16) -> Self {
        Self {
            min: value,
            max: value,
            kind: SizeKind::Fit,
        }
    }
}

impl Default for SizeAxis {
    fn default() -> Self {
        Self::FIT
    }
}
