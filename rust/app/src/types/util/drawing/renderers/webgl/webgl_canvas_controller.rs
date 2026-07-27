use std::{cell::RefCell, ops::Deref, rc::Rc};

use bon::Builder;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{
    Element, HtmlCanvasElement, ResizeObserver, ResizeObserverEntry, WebGl2RenderingContext,
};

use crate::types::util::drawing::renderers::webgl::webgl_canvas_controller::webgl_canvas_controller_builder::{IsSet, IsUnset, State};

pub struct WebglCanvasController<D> {
    inner: Rc<RefCell<WebglCanvasControllerInner<D>>>,
}
impl<D> Clone for WebglCanvasController<D> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<D> WebglCanvasController<D> {
    pub fn builder() -> WebglCanvasControllerBuilder<D> {
        WebglCanvasControllerInner::builder()
    }
}

#[derive(Builder)]
#[builder(builder_type(vis = "pub", name = WebglCanvasControllerBuilder), finish_fn(vis="", name=build_internal))]
pub struct WebglCanvasControllerInner<D> {
    #[builder(setters(vis = "", name = canvas_internal))]
    canvas: HtmlCanvasElement,
    #[builder(setters(vis = "", name = webgl_context_internal))]
    webgl_context: WebGl2RenderingContext,
    #[builder(setters(vis = "", name = context_internal))]
    context: Rc<RefCell<D>>,
    #[builder(default, setters(vis = "", name = state_internal))]
    state: CanvasState,
    /// The render function
    #[builder(with = |v: impl Fn(usize, &mut D, &WebGl2RenderingContext, CanvasSize) -> () + 'static| Box::new(v))]
    render: Box<dyn Fn(usize, &mut D, &WebGl2RenderingContext, CanvasSize) -> ()>,
    /// On click handler
    #[builder(with = |v: impl Fn(&ClickEvent, &mut D, &WebGl2RenderingContext) -> () + 'static| Box::new(v))]
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut D, &WebGl2RenderingContext) -> ()>>,
    /// On drag handler
    #[builder(with = |v: impl Fn(&DragEvent, &mut D, &WebGl2RenderingContext) -> () + 'static| Box::new(v))]
    on_drag: Option<Box<dyn Fn(&DragEvent, &mut D, &WebGl2RenderingContext) -> ()>>,
}
impl<D: 'static, S: State> WebglCanvasControllerBuilder<D, S> {
    pub fn build<F: Fn(&WebGl2RenderingContext, CanvasSize) -> D>(
        self,
        canvas: HtmlCanvasElement,
        get_context: F,
    ) -> Result<WebglCanvasController<D>, JsValue>
    where
        S::Render: IsSet,
        S::Canvas: IsUnset,
        S::WebglContext: IsUnset,
        S::Context: IsUnset,
    {
        let webgl_context = canvas
            .get_context("webgl2")?
            .ok_or(JsValue::null())?
            .dyn_into::<WebGl2RenderingContext>()?;
        let context = get_context(&webgl_context, CanvasSize::from(canvas.as_ref()));
        let inner = self
            .canvas_internal(canvas)
            .webgl_context_internal(webgl_context)
            .context_internal(Rc::new(RefCell::new(context)))
            .build_internal();
        let mut res = WebglCanvasController {
            inner: Rc::new(RefCell::new(inner)),
        };
        res.init();
        Ok(res)
    }
}

#[derive(Clone)]
pub struct ClickEvent {
    button: MouseButton,
    x: f32,
    y: f32,
    kind: EventType,
}
#[derive(Clone)]
pub struct DragEvent {
    x: f32,
    y: f32,
    start: ClickEvent,
}
#[derive(Clone)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}
#[derive(Clone)]
pub enum EventType {
    Press,
    Release { press: Box<ClickEvent> },
}

#[derive(Default)]
struct CanvasState {
    size: CanvasSize,
    render_until: usize,
    listeners: Option<CanvasListeners>,
}
struct CanvasListeners {
    size_callback: Closure<dyn FnMut(js_sys::Array, ResizeObserver)>,
    size_observer: ResizeObserver,
    render_callback: Option<Closure<dyn FnMut(f64) -> ()>>,
}
#[derive(Default, Clone, Copy)]
pub struct CanvasSize {
    pub width: usize,
    pub height: usize,
}
impl CanvasSize {
    pub fn from(element: impl Deref<Target = Element>) -> Self {
        let size = element.get_bounding_client_rect();
        CanvasSize {
            width: size.width() as usize,
            height: size.height() as usize,
        }
    }
}

impl<D: 'static> WebglCanvasController<D> {
    fn init(&mut self) {
        let mut size_clone = self.clone();
        let size_callback = Closure::<dyn FnMut(js_sys::Array<JsValue>, ResizeObserver)>::new(
            move |entries: js_sys::Array<JsValue>, _observer| {
                for entry in entries.iter() {
                    let entry: ResizeObserverEntry = entry.unchecked_into();

                    let rect = entry.content_rect();
                    let width = rect.width();
                    let height = rect.height();

                    size_clone.update_size(width as usize, height as usize);
                }
            },
        );

        let mut inner = self.inner.borrow_mut();
        let observer = ResizeObserver::new(size_callback.as_ref().unchecked_ref()).unwrap();
        let element: &Element = inner.canvas.as_ref();
        observer.observe(element);
        let size = element.get_bounding_client_rect();
        inner.state.size = CanvasSize::from(element);

        let listeners = CanvasListeners {
            size_callback,
            size_observer: observer,
            render_callback: None,
        };
        inner.state.listeners = Some(listeners);
    }
    fn update_size(&mut self, width: usize, height: usize) {
        let mut inner = self.inner.borrow_mut();
        inner.state.size.width = width;
        inner.state.size.height = height;
        drop(inner);
        self.render(0);
    }
    pub fn render(&self, duration: usize) {
        let mut inner = self.inner.borrow_mut();
        let now = js_sys::Date::now() as usize;
        inner.state.render_until = inner.state.render_until.max(now + duration);
        (*inner.render)(
            now,
            &mut inner.context.borrow_mut(),
            &inner.webgl_context,
            inner.state.size,
        );

        if inner.state.render_until > now {
            let size_clone = self.clone();
            let cb = Closure::once(move |_timestamp: f64| {
                size_clone.render(0);
            });

            web_sys::window()
                .unwrap()
                .request_animation_frame(cb.as_ref().unchecked_ref())
                .unwrap();

            if let Some(listeners) = &mut inner.state.listeners {
                listeners.render_callback = Some(cb);
            }
        }
    }
}
