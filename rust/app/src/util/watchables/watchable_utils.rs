use std::{cell::RefCell, rc::Rc};

use crate::util::watchables::{
    Constant, DataState, Listener, Observer, WatchableSetter, WatchableState,
};

use super::{derived::Derived, watchable::Watchable};

// Watch calls in both directions
pub trait Tracker<W: Watchable + ?Sized> {
    fn observe(&self, w: &W);
}
pub trait Watching<W: Watchable> {
    /// Obtains and observes the watchable value
    fn watch<T: Tracker<W>>(&self, tracker: &T) -> Rc<W::Output>;
}
impl<W: Watchable> Watching<W> for W {
    fn watch<T: Tracker<Self>>(&self, tracker: &T) -> Rc<W::Output> {
        tracker.observe(&self);
        self.get()
    }
}
pub trait Watcher<W: Watchable> {
    fn watch(&self, watchable: &W) -> Rc<W::Output>;
}
impl<W: Watchable, T: Tracker<W>> Watcher<W> for T {
    fn watch(&self, watchable: &W) -> Rc<W::Output> {
        watchable.watch(self)
    }
}

// Observer helpers
pub trait Listen<V> {
    /// Listens to updates and calls the listener with the latest value.
    /// Returns the observer that performs the observation. Once the observer is dropped, no more observations will happen
    ///
    /// Note that observing without performing `.get()` may result in no state changes occurring (see the spec)
    #[must_use = "When the observer is dropped, observation automatically stops"]
    fn listen(&self, listener: impl Fn(Rc<V>) -> () + 'static) -> Observer;

    /// Listens to updates and calls the listener with the latest value.
    /// Returns the observer that performs the observation. Once the observer is dropped, no more observations will happen
    ///
    /// Note that observing without performing `.get()` may result in no state changes occurring (see the spec)
    #[must_use = "When the observer is dropped, observation automatically stops"]
    fn listen_mut(&self, listener: impl FnMut(Rc<V>) -> () + 'static) -> Observer;
}
impl<W: Watchable + Clone + 'static> Listen<W::Output> for W {
    fn listen(&self, listener: impl Fn(Rc<W::Output>) -> () + 'static) -> Observer {
        let watchable = self.clone();
        self.observe(Box::new(move || listener(watchable.get())))
    }

    fn listen_mut(&self, mut listener: impl FnMut(Rc<W::Output>) -> () + 'static) -> Observer {
        let watchable = self.clone();
        self.observe(Box::new(RefCell::new(move || listener(watchable.get()))))
    }
}

impl<D: Fn() -> ()> Listener for D {
    fn state_changed(&self, state: DataState) {
        if state == DataState::UpToDate {
            (self)();
        }
    }
}

impl<D: FnMut() -> ()> Listener for RefCell<D> {
    fn state_changed(&self, state: DataState) {
        if state == DataState::UpToDate {
            (self.borrow_mut())();
        }
    }
}

// Watchable modifiers
pub trait WatchableUtils<X> {
    fn map<Y: 'static, F: Fn(Rc<X>) -> Y + 'static>(self, map: F) -> Derived<Y>;
}
impl<X, W: Watchable<Output = X> + 'static> WatchableUtils<X> for W {
    fn map<Y: 'static, F: Fn(Rc<X>) -> Y + 'static>(self, map: F) -> Derived<Y> {
        Derived::new(move |t| {
            t.observe(&self);
            map(self.get())
        })
    }
}

pub trait CloneableWatchableUtils<X: Clone> {
    fn option(self) -> Derived<Option<X>>;
}
impl<X: Clone, W: Watchable<Output = X> + 'static> CloneableWatchableUtils<X> for W {
    fn option(self) -> Derived<Option<X>> {
        self.map(|v| Some((*v).clone()))
    }
}

// Into watchables

pub trait IntoWatchable<X> {
    type Output: Watchable<Output = X>;
    fn into_watchable(self) -> Self::Output;
}
pub trait IntoWatchableSetter<X> {
    type Output: WatchableSetter<Output = X>;
    fn into_watchable_setter(self) -> Self::Output;
}

impl IntoWatchable<String> for &str {
    type Output = Constant<String>;

    fn into_watchable(self) -> Self::Output {
        Constant::new(self.to_string())
    }
}
impl IntoWatchable<Option<String>> for &str {
    type Output = Constant<Option<String>>;

    fn into_watchable(self) -> Self::Output {
        Constant::new(Some(self.to_string()))
    }
}

// Change testers
pub struct Changed<W: Watchable>
where
    W::Output: Eq,
{
    watchable: W,
    prev: RefCell<Rc<W::Output>>,
    true_f: Rc<bool>,
    false_f: Rc<bool>,
}
impl<W: Watchable> Changed<W>
where
    W::Output: Eq,
{
    pub fn new(watchable: W) -> Self {
        let val = watchable.get();
        Changed {
            watchable,
            prev: RefCell::new(val),
            true_f: Rc::new(true),
            false_f: Rc::new(false),
        }
    }
}
impl<W: Watchable> WatchableState for Changed<W>
where
    W::Output: Eq,
{
    fn state(&self) -> DataState {
        self.watchable.state()
    }

    fn observe(&self, listener: Box<dyn Listener>) -> Observer {
        self.watchable.observe(listener)
    }
}
impl<W: Watchable> Watchable for Changed<W>
where
    W::Output: Eq,
{
    type Output = bool;
    fn get(&self) -> Rc<Self::Output> {
        let new_value = self.watchable.get();
        let changed = new_value != *self.prev.borrow();
        *self.prev.borrow_mut() = new_value;
        match changed {
            true => self.true_f.clone(),
            false => self.false_f.clone(),
        }
    }
}
