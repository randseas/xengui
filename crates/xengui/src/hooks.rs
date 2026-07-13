// SPDX-License-Identifier: Apache-2.0
use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use winit::window::Window;

thread_local! {
    static HOOK_STORE: RefCell<HookStore> = RefCell::new(HookStore::new());
    static REDRAW_HANDLE: RefCell<Option<Arc<Window>>> = const { RefCell::new(None) };
}

struct HookStore {
    slots: Vec<Rc<RefCell<Box<dyn Any>>>>,
    cursor: usize,
    dirty: bool,
}

impl HookStore {
    fn new() -> Self {
        Self {
            slots: Vec::new(),
            cursor: 0,
            dirty: false,
        }
    }
}

pub(crate) fn begin_render() {
    HOOK_STORE.with(|s| {
        s.borrow_mut().cursor = 0;
    });
}

pub(crate) fn take_dirty() -> bool {
    HOOK_STORE.with(|s| std::mem::take(&mut s.borrow_mut().dirty))
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

/// useState(initial).
///
/// initial is used ONLY during the initial render; in subsequent renders, the
/// existing slot value is preserved as-is.
///
/// Returns: (current_value, set_function). SetState<T> is Clone and
/// 'static - it can be freely moved into closures like on_click.
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
    HOOK_STORE.with(|store| {
        let idx;
        let slot;
        {
            let mut store = store.borrow_mut();
            idx = store.cursor;
            store.cursor += 1;

            if idx == store.slots.len() {
                store.slots.push(Rc::new(RefCell::new(Box::new(initial) as Box<dyn Any>)));
            }
            slot = store.slots[idx].clone();
        }

        let value = slot
            .borrow()
            .downcast_ref::<T>()
            .expect(
                "use_state: hook order changed between renders - do not call use_state conditionally (inside if/loop)"
            )
            .clone();

        (
            value,
            SetState {
                slot,
                _marker: PhantomData,
            },
        )
    })
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
        HOOK_STORE.with(|s| {
            s.borrow_mut().dirty = true;
        });
        request_redraw();
    }

    pub fn update(&self, f: impl FnOnce(&T) -> T) {
        let new_value = {
            let borrowed = self.slot.borrow();
            let current = borrowed
                .downcast_ref::<T>()
                .expect("use_state: SetState<T> used with wrong type");
            f(current)
        };
        self.set(new_value);
    }
}
