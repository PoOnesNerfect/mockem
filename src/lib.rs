//! Mock any function types in Rust.
//!
//! Make sure to only use this crate for testing purposes, as it will add a lot of overhead to your code.
//!
//! ## Basic Usage
//!
//! On the function you want to mock, add the `#[mock]` attribute.
//!
//! ```rust
//! #[cfg_attr(test, mockem::mock)]
//! fn foo() -> String {
//!     format!("foo")
//! }
//!
//! fn bar() -> String {
//!     format!("Hello, {}!", foo())
//! }
//!
//! #[test]
//! fn test_fn() {
//!     use mockem::MockCall;
//!
//!     foo.mock_ret("mockem".to_owned());
//!     foo.mock_continue();
//!     foo.mock_ret("mockem2".to_owned());
//!
//!     assert_eq!(&bar(), "Hello, mockem!");
//!     assert_eq!(&bar(), "Hello, foo!");
//!     assert_eq!(&bar(), "Hello, mockem2!");
//!     assert_eq!(&bar(), "Hello, foo!");
//! }
//! ```
//!
//! ## Impl Blocks
//!
//! If you want to mock impl methods, add the `#[mock]` attribute to the impl block.
//! Do the same for impl trait methods.
//!
//! This will mock all methods in the impl block.
//!
//! ```rust
//! struct Foo;
//!
//! #[cfg_attr(test, mockem::mock)]
//! impl Foo {
//!     fn foo(&self) -> String {
//!         format!("foo")
//!     }
//! }
//!
//! trait Baz {
//!     fn baz(&self) -> String;
//! }
//!
//! #[cfg_attr(test, mockem::mock)]
//! impl Baz for Foo {
//!     fn baz(&self) -> String {
//!         format!("baz")
//!     }
//! }
//!
//! fn bar() -> String {
//!     format!("Hello, {} and {}!", Foo.foo(), Foo.baz())
//! }
//!
//! #[test]
//! fn test_fn() {
//!     use mockem::MockCall;
//!
//!     Foo::foo.mock_ret("mockem".to_owned());
//!     Foo::baz.mock_ret("mockem2".to_owned());
//!
//!     assert_eq!(&bar(), "Hello, mockem and mockem2!");
//! }
//! ```
//!
//! ## Async Functions
//!
//! Async functions are also supported.
//!
//! ```rust
//! use async_trait::async_trait;
//!
//! struct Foo;
//!
//! #[cfg_attr(test, mockem::mock)]
//! impl Foo {
//!     async fn foo(&self) -> String {
//!         format!("foo")
//!     }
//! }
//!
//! #[async_trait]
//! trait Baz {
//!     async fn baz(&self) -> String;
//! }
//!
//! // also works with async_trait
//! // but you must place #[mock] above #[async_trait]
//! #[cfg_attr(test, mockem::mock)]
//! #[async_trait]
//! impl Baz for Foo {
//!     async fn baz(&self) -> String {
//!         format!("baz")
//!     }
//! }
//!
//! async fn bar() -> String {
//!     format!("Hello, {} and {}!", Foo.foo().await, Foo.baz().await)
//! }
//!
//! #[test]
//! fn test_fn() {
//!     use mockem::MockCall;
//!
//!     Foo::foo.mock_ret("mockem".to_owned());
//!     Foo::baz.mock_ret("mockem2".to_owned());
//!
//!     assert_eq!(&bar().await, "Hello, mockem and mockem2!");
//! }
//! ```

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, VecDeque},
    future::Future,
    rc::Rc,
};

pub use mockem_derive::mock;

thread_local! {
    static MOCK_STORE: MockStore = MockStore::default()
}

/// Clear all mocks in the ThreadLocal; only necessary if tests share threads
pub fn clear_mocks() {
    MOCK_STORE.with(|mock_store| mock_store.clear())
}

#[doc(hidden)]
pub enum MockReturn {
    Ret(Rc<dyn Any>),
    Continue,
}

impl MockReturn {
    pub fn new(ret: impl Any) -> Self {
        Self::Ret(Rc::new(ret))
    }
}

#[doc(hidden)]
#[derive(Default)]
pub struct MockStore {
    // (fn type_id) -> return_value
    mocks: RefCell<HashMap<TypeId, VecDeque<MockReturn>>>,
}

impl MockStore {
    fn clear(&self) {
        self.mocks.borrow_mut().clear()
    }

    pub unsafe fn add(&self, id: TypeId, value: MockReturn) {
        {
            if let Some(returns) = self.mocks.borrow_mut().get_mut(&id) {
                returns.push_back(value);
                return;
            }
        }

        self.mocks.borrow_mut().insert(id, vec![value].into());
    }

    fn get(&self, id: TypeId) -> Option<MockReturn> {
        self.mocks
            .borrow_mut()
            .get_mut(&id)
            .and_then(|returns| returns.pop_front())
    }
}

/// Auto-implemented trait for mocking return values of functions.
///
/// Works for:
/// - functions/methods,
/// - async functions/methods,
/// - trait methods, and
/// - async_trait methods.
///
/// The trait is implemented for functions with up to 12 arguments.
pub trait MockCall<T, O: 'static, Fut> {
    fn mock_ret(&self, ret: O);
    fn mock_continue(&self);

    fn call_mock(&self) -> Option<O> {
        let id = self.get_mock_id();
        if let Some(MockReturn::Ret(o)) = MOCK_STORE.with(|mock_store| mock_store.get(id)) {
            return Rc::downcast::<O>(o)
                .ok()
                .map(|o| Rc::into_inner(o))
                .flatten();
        } else {
            return None;
        }
    }

    fn get_mock_id(&self) -> TypeId {
        (|| ()).type_id()
    }
}

#[doc(hidden)]
pub struct __NotFuture;

impl<O: Any, F: Fn() -> O + 'static> MockCall<(), O, __NotFuture> for F {
    fn mock_ret(&self, ret: O) {
        unsafe {
            MOCK_STORE.with(|mock_store| {
                mock_store.add(self.get_mock_id(), MockReturn::new(ret));
            });
        }
    }

    fn mock_continue(&self) {
        unsafe {
            MOCK_STORE.with(|mock_store| {
                mock_store.add(self.get_mock_id(), MockReturn::Continue);
            });
        }
    }
}

impl<O: Any, F: Fn() -> Fut + 'static, Fut: Future<Output = O>> MockCall<(), O, Fut> for F {
    fn mock_ret(&self, ret: O) {
        unsafe {
            MOCK_STORE.with(|mock_store| {
                mock_store.add(
                    <Self as MockCall<(), O, Fut>>::get_mock_id(self),
                    MockReturn::new(ret),
                );
            });
        }
    }

    fn mock_continue(&self) {
        unsafe {
            MOCK_STORE.with(|mock_store| {
                mock_store.add(
                    <Self as MockCall<(), O, Fut>>::get_mock_id(self),
                    MockReturn::Continue,
                );
            });
        }
    }
}

#[rustfmt::skip]
macro_rules! all_the_tuples {
    ($name:ident) => {
        $name!(T1);
        $name!(T1, T2);
        $name!(T1, T2, T3);
        $name!(T1, T2, T3, T4);
        $name!(T1, T2, T3, T4, T5);
        $name!(T1, T2, T3, T4, T5, T6);
        $name!(T1, T2, T3, T4, T5, T6, T7);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
    };
}

macro_rules! impl_mock_call {
    ($($T:ident),*) => {
        impl<$($T),*, O: Any, F: Fn($($T),*) -> O + 'static> MockCall<($($T,)*), O, __NotFuture> for F {
            fn mock_ret(&self, ret: O) {
                unsafe {
                    MOCK_STORE.with(|mock_store| {
                        mock_store.add(self.get_mock_id(), MockReturn::new(ret));
                    });
                }
            }

            fn mock_continue(&self) {
                unsafe {
                    MOCK_STORE.with(|mock_store| {
                        mock_store.add(self.get_mock_id(), MockReturn::Continue);
                    });
                }
            }
        }
    }
}

all_the_tuples!(impl_mock_call);

macro_rules! impl_mock_async_call {
    ($($T:ident),*) => {
        impl<$($T),*, O: Any, F: Fn($($T),*) -> Fut, Fut: Future<Output = O>> MockCall<($($T,)*), O, Fut> for F {
            fn mock_ret(&self, ret: O) {
                unsafe {
                    MOCK_STORE.with(|mock_store| {
                        mock_store.add(<Self as MockCall<_, O, Fut>>::get_mock_id(self), MockReturn::new(ret));
                    });
                }
            }

            fn mock_continue(&self) {
                unsafe {
                    MOCK_STORE.with(|mock_store| {
                        mock_store.add(<Self as MockCall<_, O, Fut>>::get_mock_id(self), MockReturn::Continue);
                    });
                }
            }
        }
    }
}

all_the_tuples!(impl_mock_async_call);
