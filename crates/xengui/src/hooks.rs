// SPDX-License-Identifier: Apache-2.0
use smol_str::SmolStr;
use std::any::Any;
use std::cell::{ Cell, RefCell };
use std::collections::{ HashMap, HashSet };
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use winit::window::Window;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComponentId(SmolStr);

impl ComponentId {
    pub fn root() -> Self {
        Self(SmolStr::new("root"))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Clone, Debug)]
pub struct ComponentKey(SmolStr);

impl From<&str> for ComponentKey {
    fn from(v: &str) -> Self {
        Self(SmolStr::new(v))
    }
}

impl From<String> for ComponentKey {
    fn from(v: String) -> Self {
        Self(SmolStr::new(v))
    }
}

impl From<SmolStr> for ComponentKey {
    fn from(v: SmolStr) -> Self {
        Self(v)
    }
}

macro_rules! impl_component_key_from_int {
    ($($t:ty),*) => {
        $(
            impl From<$t> for ComponentKey {
                fn from(v: $t) -> Self {
                    Self(SmolStr::new(v.to_string()))
                }
            }
        )*
    };
}
impl_component_key_from_int!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

struct ComponentState {
    slots: Vec<Rc<RefCell<Box<dyn Any>>>>,
    cursor: usize,
}

impl ComponentState {
    fn new() -> Self {
        Self { slots: Vec::new(), cursor: 0 }
    }
}

thread_local! {
    static HOOK_STORE: RefCell<HashMap<ComponentId, ComponentState>> = RefCell::new(HashMap::new());

    static COMPONENT_STACK: RefCell<Vec<ComponentId>> = const { RefCell::new(Vec::new()) };

    static LIVE_COMPONENTS: RefCell<HashSet<ComponentId>> = RefCell::new(HashSet::new());

    static DIRTY: Cell<bool> = const { Cell::new(false) };
    static REDRAW_HANDLE: RefCell<Option<Arc<Window>>> = const { RefCell::new(None) };
}

pub(crate) fn begin_render() {
    LIVE_COMPONENTS.with(|s| s.borrow_mut().clear());
    COMPONENT_STACK.with(|s| {
        let mut s = s.borrow_mut();
        debug_assert!(
            s.is_empty(),
            "xengui hooks: component stack boş değil - begin_render/end_render dengesiz çağrılmış olabilir"
        );
        s.clear();
    });
}

pub(crate) fn end_render() {
    LIVE_COMPONENTS.with(|live| {
        let live = live.borrow();
        HOOK_STORE.with(|store| {
            store.borrow_mut().retain(|id, _| live.contains(id));
        });
    });
}

pub(crate) fn take_dirty() -> bool {
    DIRTY.with(|d| d.replace(false))
}

pub(crate) fn set_redraw_handle(window: Arc<Window>) {
    REDRAW_HANDLE.with(|h| {
        *h.borrow_mut() = Some(window);
    });
}

fn request_redraw() {
    REDRAW_HANDLE.with(|h| {
        if let Some(window) = h.borrow().as_ref() {
            window.request_redraw();
        }
    });
}

fn current_component_id() -> ComponentId {
    COMPONENT_STACK.with(|s| {
        s.borrow()
            .last()
            .cloned()
            .unwrap_or_else(|| {
                panic!(
                    "use_state: bir component() kapsamı dışında çağrıldı. \
                 use_state yalnızca App::render kök fonksiyonu içinde veya \
                 component(key, ...) kapsamı içinde kullanılabilir."
                )
            })
    })
}

fn push_component(key: ComponentKey) -> ComponentId {
    let id = COMPONENT_STACK.with(|s| {
        match s.borrow().last() {
            Some(parent) =>
                ComponentId(SmolStr::new(format!("{}\u{1f}{}", parent.as_str(), key.0))),
            None => ComponentId(key.0),
        }
    });

    HOOK_STORE.with(|store| {
        let mut store = store.borrow_mut();
        let state = store.entry(id.clone()).or_insert_with(ComponentState::new);
        state.cursor = 0;
    });

    let first_time_this_frame = LIVE_COMPONENTS.with(|s| s.borrow_mut().insert(id.clone()));
    if !first_time_this_frame {
        log::warn!(
            "xengui: yinelenen bileşen anahtarı '{}' - aynı karede iki kez kullanıldı. \
             Dinamik listelerde her öğeye benzersiz bir key verin (React'taki 'key' prop'u gibi).",
            id.as_str()
        );
    }

    COMPONENT_STACK.with(|s| s.borrow_mut().push(id.clone()));
    id
}

fn pop_component() {
    COMPONENT_STACK.with(|s| {
        s.borrow_mut().pop();
    });
}

/// component(key, render).
///
/// ```ignore
/// let mut list_view = View::new().flex_direction(FlexDirection::Column);
/// for item in &items {
///     list_view = list_view.child(component(item.id, || {
///         let (checked, set_checked) = use_state(false);
///         Button::new()
///             .label(if checked { "✓" } else { "" })
///             .on_click(move |_ctx| set_checked.set(!checked))
///     }));
/// }
/// ```
pub fn component<R>(key: impl Into<ComponentKey>, render: impl FnOnce() -> R) -> R {
    push_component(key.into());
    let result = render();
    pop_component();
    result
}

/// useState(initial).
///
/// ```ignore
/// let (count, set_count) = use_state(0i32);
///
/// View::new().child(
///     Button::new()
///         .label(format!("Count: {count}"))
///         .on_click(move |_ctx| set_count.set(count + 1))
/// )
/// ```
pub fn use_state<T: Clone + 'static>(initial: T) -> (T, SetState<T>) {
    let id = current_component_id();

    let (slot, idx) = HOOK_STORE.with(|store| {
        let mut store = store.borrow_mut();
        let state = store
            .get_mut(&id)
            .expect("use_state: internal error - provided binding used without begin/push");

        let idx = state.cursor;
        state.cursor += 1;

        if idx == state.slots.len() {
            state.slots.push(Rc::new(RefCell::new(Box::new(initial) as Box<dyn Any>)));
        }

        (state.slots[idx].clone(), idx)
    });

    let value = {
        let borrowed = slot.borrow();
        borrowed
            .downcast_ref::<T>()
            .unwrap_or_else(|| {
                panic!(
                    "use_state: hook order broken in component '{}' (slot #{idx}) - do not call use_state conditionally (inside an if/loop). In dynamic lists, wrap each item in a component (e.g., component(key, ...)) to give it its own isolated hook order.",
                    id.as_str()
                )
            })
            .clone()
    };

    (
        value,
        SetState {
            slot,
            _marker: PhantomData,
        },
    )
}

pub struct SetState<T> {
    slot: Rc<RefCell<Box<dyn Any>>>,
    _marker: PhantomData<T>,
}

impl<T> Clone for SetState<T> {
    fn clone(&self) -> Self {
        Self {
            slot: self.slot.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: 'static> SetState<T> {
    pub fn set(&self, value: T) {
        *self.slot.borrow_mut() = Box::new(value);
        DIRTY.with(|d| d.set(true));
        request_redraw();
    }

    pub fn update(&self, f: impl FnOnce(&T) -> T) {
        let new_value = {
            let borrowed = self.slot.borrow();
            let current = borrowed
                .downcast_ref::<T>()
                .expect("use_state: SetState<T> used with the wrong type");
            f(current)
        };
        self.set(new_value);
    }
}
