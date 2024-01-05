//! Mock any function types in Rust.
//!
//! Make sure to only use this crate for testing purposes, as it will add a lot of overhead to your code.
//! `.mock_once(..)` expects a closure that takes the arguments of the function and returns the same return type as the function.
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
//!     foo.mock_once(|| "mockem".to_owned());
//!     foo.mock_once(|| "mockem2".to_owned());
//!
//!     assert_eq!(&bar(), "Hello, mockem!");
//!     assert_eq!(&bar(), "Hello, mockem2!");
//!
//!     // works normally after all mocks are used
//!     assert_eq!(&bar(), "Hello, foo!");
//! }
//! ```
//!
//! ### Mocking Repeatedly
//!
//! If you want to mock a function more than once or indefinitely, use `mock_repeat` instead of `mock_once`.
//!
//! `mock_repeat` takes an `Option<usize>` as its first argument, which is the number of times to mock the function;
//!
//! `None` means to mock the function indefinitely.
//!
//!
//! ```rust
//! #[cfg_attr(test, mockem::mock)]
//! fn foo(a: &str) -> String {
//!     format!("{a}")
//! }
//!
//! fn bar(a: &str) -> String {
//!     format!("Hello, {}!", foo(a))
//! }
//!
//! #[test]
//! fn test_fn() {
//!     use mockem::{MockCall, ClearMocks};
//!
//!     foo.mock_repeat(None, |a| format!("mocked {a}"));
//!
//!     assert_eq!(&bar("bar"), "Hello, mocked bar!");
//!     assert_eq!(&bar("foo"), "Hello, mocked foo!");
//!     assert_eq!(&bar("baz"), "Hello, mocked baz!");
//!
//!     // this clears all mocks, which will stop the indefinite mock
//!     foo.clear_mocks();
//!
//!     assert_eq!(&bar("baz"), "Hello, baz!");
//! }
//! ```
//!
//!
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
//!     Foo::foo.mock_once(|_| "mockem".to_owned());
//!     Foo::baz.mock_once(|_| "mockem2".to_owned());
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
//!     Foo::foo.mock_once(|_| "mockem".to_owned());
//!     Foo::baz.mock_once(|_| "mockem2".to_owned());
//!
//!     assert_eq!(&bar().await, "Hello, mockem and mockem2!");
//! }
//! ```

use std::{
    any::{Any, TypeId},
    future::Future,
    marker::PhantomData,
    mem::transmute,
    rc::Rc,
};

mod store;
use store::MockStore;

pub use mockem_derive::mock;

thread_local! {
    static MOCK_STORE: MockStore = MockStore::default()
}

/// Clear all mocks in the ThreadLocal; only necessary if tests share threads
pub fn clear_mocks() {
    MOCK_STORE.with(|mock_store| mock_store.clear())
}

#[doc(hidden)]
pub struct MockReturn(Rc<Box<dyn FnMut() -> ()>>, Option<usize>);

/// Auto-implemented trait for mocking return values of functions.
///
/// Works for:
/// - functions/methods,
/// - async functions/methods,
/// - trait methods, and
/// - async_trait methods.
///
/// The trait is implemented for functions with up to 12 arguments.
pub trait MockCall<I, O, W, Fut>: CallMock<I, O, Fut> {
    /// Mock the return value of this function.
    /// This expects a closure with the arguments of the function.
    fn mock_once(&self, with: W);
    fn mock_repeat(&self, repeat: Option<usize>, with: W);
}

/// Clear all mocked return values related to this function.
/// You can use this if you have a recursive mock closure that continously mocks.
pub trait ClearMocks<I, O, Fut>: CallMock<I, O, Fut> {
    fn clear_mocks(&self) {
        let id = self.get_mock_id();

        MOCK_STORE.with(|mock_store| mock_store.remove(id));
    }
}
impl<I, O, Fut, F: CallMock<I, O, Fut>> ClearMocks<I, O, Fut> for F {}

#[doc(hidden)]
pub trait CallMock<I, O, Fut> {
    fn mock_exists(&self, _ret: PhantomData<O>) -> bool {
        let id = self.get_mock_id();

        MOCK_STORE.with(|mock_store| mock_store.mock_exists(id))
    }

    fn call_mock(&self, input: I) -> O;

    fn get_mock_id(&self) -> TypeId {
        (|| ()).type_id()
    }
}

#[doc(hidden)]
pub struct NotFuture;

impl<O, W: FnMut() -> O + 'static, F: Fn() -> O> MockCall<(), O, W, NotFuture> for F {
    fn mock_once(&self, with: W) {
        self.mock_repeat(Some(1), with)
    }

    fn mock_repeat(&self, repeat: Option<usize>, with: W) {
        let erased: Box<dyn FnMut() -> O + 'static> = Box::new(with);
        let transmuted: Rc<Box<dyn FnMut() -> ()>> = unsafe { transmute(Rc::new(erased)) };

        MOCK_STORE.with(|mock_store| {
            mock_store.add(self.get_mock_id(), MockReturn(transmuted, repeat));
        });
    }
}

impl<O, F: Fn() -> O> CallMock<(), O, NotFuture> for F {
    fn call_mock(&self, _: ()) -> O {
        let id = self.get_mock_id();

        if let Some(MockReturn(with, repeat)) = MOCK_STORE.with(|mock_store| mock_store.get(id)) {
            let with: Rc<Box<dyn FnMut() -> O + 'static>> = unsafe { transmute(with) };
            let mut boxed = Rc::into_inner(with).expect("mock should exist");
            let ret = boxed();

            if let Some(repeat) = repeat {
                if repeat > 1 {
                    let transmuted: Rc<Box<dyn FnMut() -> ()>> =
                        unsafe { transmute(Rc::new(boxed)) };
                    MOCK_STORE.with(|mock_store| {
                        mock_store
                            .add(self.get_mock_id(), MockReturn(transmuted, Some(repeat - 1)));
                    });
                }
            } else {
                let transmuted: Rc<Box<dyn FnMut() -> ()>> = unsafe { transmute(Rc::new(boxed)) };
                MOCK_STORE.with(|mock_store| {
                    mock_store.add(self.get_mock_id(), MockReturn(transmuted, None));
                });
            }

            ret
        } else {
            panic!("mock should exist")
        }
    }
}

impl<O, W: FnMut() -> O + 'static, F: Fn() -> Fut + 'static, Fut: Future<Output = O>>
    MockCall<(), O, W, Fut> for F
{
    fn mock_once(&self, f: W) {
        self.mock_repeat(Some(1), f)
    }

    fn mock_repeat(&self, repeat: Option<usize>, with: W) {
        let erased: Box<dyn FnMut() -> O + 'static> = Box::new(with);
        let transmuted: Rc<Box<dyn FnMut() -> ()>> = unsafe { transmute(Rc::new(erased)) };

        MOCK_STORE.with(|mock_store| {
            mock_store.add(
                <Self as CallMock<(), O, Fut>>::get_mock_id(self),
                MockReturn(transmuted, repeat),
            );
        });
    }
}

impl<O, F: Fn() -> Fut, Fut: Future<Output = O>> CallMock<(), O, Fut> for F {
    fn call_mock(&self, _: ()) -> O {
        let id = <Self as CallMock<(), O, Fut>>::get_mock_id(self);

        if let Some(MockReturn(with, repeat)) = MOCK_STORE.with(|mock_store| mock_store.get(id)) {
            let with: Rc<Box<dyn FnMut() -> O + 'static>> = unsafe { transmute(with) };
            let mut boxed = Rc::into_inner(with).expect("mock should exist");
            let ret = boxed();

            if let Some(repeat) = repeat {
                if repeat > 1 {
                    let transmuted: Rc<Box<dyn FnMut() -> ()>> =
                        unsafe { transmute(Rc::new(boxed)) };
                    MOCK_STORE.with(|mock_store| {
                        mock_store.add(
                            <Self as CallMock<(), O, Fut>>::get_mock_id(self),
                            MockReturn(transmuted, Some(repeat - 1)),
                        );
                    });
                }
            } else {
                let transmuted: Rc<Box<dyn FnMut() -> ()>> = unsafe { transmute(Rc::new(boxed)) };
                MOCK_STORE.with(|mock_store| {
                    mock_store.add(
                        <Self as CallMock<(), O, Fut>>::get_mock_id(self),
                        MockReturn(transmuted, None),
                    );
                });
            }

            ret
        } else {
            panic!("mock should exist")
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
        impl<$($T),*, O, W: FnMut($($T),*) -> O + 'static, F: Fn($($T),*) -> O> MockCall<($($T,)*), O, W, NotFuture>
            for F
        {
            fn mock_once(&self, f: W) {
                self.mock_repeat(Some(1), f)
            }

            fn mock_repeat(&self, repeat: Option<usize>, with: W) {
                let erased: Box<dyn FnMut($($T),*) -> O + 'static> = Box::new(with);
                let transmuted: Rc<Box<dyn FnMut() -> ()>> = unsafe { transmute(Rc::new(erased)) };

                MOCK_STORE.with(|mock_store| {
                    mock_store.add(
                        <Self as CallMock<($($T,)*), O, NotFuture>>::get_mock_id(self),
                        MockReturn(transmuted, repeat),
                    );
                });
            }
        }

        impl<$($T),*, O, F: Fn($($T),*) -> O> CallMock<($($T,)*), O, NotFuture>
            for F
        {
            #[allow(non_snake_case)]
            fn call_mock(&self, ($($T,)*): ($($T,)*)) -> O {
                let id = <Self as CallMock<($($T,)*), O, NotFuture>>::get_mock_id(self);

                if let Some(MockReturn(with, repeat)) = MOCK_STORE.with(|mock_store| mock_store.get(id)) {
                    let with: Rc<Box<dyn FnMut($($T),*) -> O + 'static>> = unsafe { transmute(with) };
                    let mut boxed = Rc::into_inner(with).expect("mock should exist");
                    let ret = boxed($($T),*);

                    if let Some(repeat) = repeat {
                        if repeat > 1 {
                            let transmuted: Rc<Box<dyn FnMut() -> ()>> =
                                unsafe { transmute(Rc::new(boxed)) };
                            MOCK_STORE.with(|mock_store| {
                                mock_store.add(
                                    <Self as CallMock<($($T,)*), O, NotFuture>>::get_mock_id(self),
                                    MockReturn(transmuted, Some(repeat - 1)),
                                );
                            });
                        }
                    } else {
                        let transmuted: Rc<Box<dyn FnMut() -> ()>> = unsafe { transmute(Rc::new(boxed)) };
                        MOCK_STORE.with(|mock_store| {
                            mock_store.add(
                                <Self as CallMock<($($T,)*), O, NotFuture>>::get_mock_id(self),
                                MockReturn(transmuted, None),
                            );
                        });
                    }

                    ret
                } else {
                    panic!("mock should exist")
                }
            }
        }
    }
}
all_the_tuples!(impl_mock_call);

macro_rules! impl_mock_async_call {
    ($($T:ident),*) => {
        impl<$($T),*, O, W: FnMut($($T),*) -> O + 'static, F: Fn($($T),*) -> Fut, Fut: Future<Output = O>> MockCall<($($T,)*), O, W, Fut>
            for F
        {
            fn mock_once(&self, f: W) {
                self.mock_repeat(Some(1), f)
            }

            fn mock_repeat(&self, repeat: Option<usize>, with: W) {
                let erased: Box<dyn FnMut($($T),*) -> O + 'static> = Box::new(with);
                let transmuted: Rc<Box<dyn FnMut() -> ()>> = unsafe { transmute(Rc::new(erased)) };

                MOCK_STORE.with(|mock_store| {
                    mock_store.add(
                        <Self as CallMock<($($T,)*), O, Fut>>::get_mock_id(self),
                        MockReturn(transmuted, repeat),
                    );
                });
            }
        }

        impl<$($T),*, O, F: Fn($($T),*) -> Fut, Fut: Future<Output = O>> CallMock<($($T,)*), O, Fut>
            for F
        {
            #[allow(non_snake_case)]
            fn call_mock(&self, ($($T,)*): ($($T,)*)) -> O {
                let id = <Self as CallMock<($($T,)*), O, Fut>>::get_mock_id(self);

                if let Some(MockReturn(with, repeat)) = MOCK_STORE.with(|mock_store| mock_store.get(id)) {
                    let with: Rc<Box<dyn FnMut($($T),*) -> O + 'static>> = unsafe { transmute(with) };
                    let mut boxed = Rc::into_inner(with).expect("mock should exist");
                    let ret = boxed($($T),*);

                    if let Some(repeat) = repeat {
                        if repeat > 1 {
                            let transmuted: Rc<Box<dyn FnMut() -> ()>> =
                                unsafe { transmute(Rc::new(boxed)) };
                            MOCK_STORE.with(|mock_store| {
                                mock_store.add(
                                    <Self as CallMock<($($T,)*), O, Fut>>::get_mock_id(self),
                                    MockReturn(transmuted, Some(repeat - 1)),
                                );
                            });
                        }
                    } else {
                        let transmuted: Rc<Box<dyn FnMut() -> ()>> = unsafe { transmute(Rc::new(boxed)) };
                        MOCK_STORE.with(|mock_store| {
                            mock_store.add(
                                <Self as CallMock<($($T,)*), O, Fut>>::get_mock_id(self),
                                MockReturn(transmuted, None),
                            );
                        });
                    }

                    ret
                } else {
                    panic!("mock should exist")
                }
            }
        }
    }
}
all_the_tuples!(impl_mock_async_call);
