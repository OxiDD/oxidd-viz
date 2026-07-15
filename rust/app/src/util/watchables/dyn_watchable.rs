use std::rc::Rc;

use crate::util::watchables::{
    DataState, IntoWatchable, Listener, Observer, Watchable, WatchableState,
};

pub struct DynWatchable<X>(Rc<dyn Watchable<Output = X>>);
impl<X> Clone for DynWatchable<X> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<X> DynWatchable<X> {
    pub fn new<W: Watchable<Output = X> + 'static>(watchable: W) -> Self {
        DynWatchable(Rc::new(watchable))
    }
    pub fn from<W: IntoWatchable<X> + 'static>(val: W) -> Self
    where
        W::Output: 'static,
    {
        Self::new(val.into_watchable())
    }
}
impl<X> Watchable for DynWatchable<X> {
    type Output = X;

    fn get(&self) -> Rc<Self::Output> {
        self.0.get()
    }
}
impl<X> WatchableState for DynWatchable<X> {
    fn state(&self) -> DataState {
        self.0.state()
    }

    fn observe(&self, listener: Box<dyn Listener>) -> Observer {
        self.0.observe(listener)
    }
}

impl<X> IntoWatchable<X> for DynWatchable<X> {
    type Output = DynWatchable<X>;
    fn into_watchable(self) -> Self::Output {
        self
    }
}
