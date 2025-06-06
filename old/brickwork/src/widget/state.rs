use tokio::sync::watch;

#[derive(Debug)]
pub struct ValueProvider<T>(watch::Sender<T>);

#[derive(Debug, Clone)]
pub struct SubscribedValue<T>(watch::Receiver<T>);

#[derive(Debug, Clone)]
pub enum WidgetValue<T> {
    Fixed(T),
    Subscribed(SubscribedValue<T>),
}

impl<T> ValueProvider<T> {
    pub fn new(value: T) -> Self {
        Self(watch::Sender::new(value))
    }

    pub fn subscribe(&self) -> SubscribedValue<T> {
        SubscribedValue(self.0.subscribe())
    }

    pub fn set(&self, value: T) {
        self.0.send_replace(value);
    }

    pub fn get(&self) -> watch::Ref<'_, T> {
        self.0.borrow()
    }

    pub fn modify(&self, modify: impl FnOnce(&mut T)) {
        self.0.send_modify(modify);
    }
}

impl<T> SubscribedValue<T> {
    pub fn is_dirty(&self) -> bool {
        self.0.has_changed().unwrap_or(false)
    }

    pub fn get_and_reset(&mut self) -> watch::Ref<'_, T> {
        self.0.borrow_and_update()
    }
}

impl<T> WidgetValue<T> {
    pub fn is_dirty(&self) -> bool {
        match self {
            WidgetValue::Fixed(_) => false,
            WidgetValue::Subscribed(subscribed) => subscribed.is_dirty(),
        }
    }

    pub fn access_and_reset<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        match self {
            WidgetValue::Fixed(value) => f(value),
            WidgetValue::Subscribed(subscribed) => f(&subscribed.get_and_reset()),
        }
    }
}
