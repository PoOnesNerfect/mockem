# Mock'em

[<img alt="github" src="https://img.shields.io/badge/github-poonesnerfect/mockem-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/poonesnerfect/mockem)
[<img alt="crates.io" src="https://img.shields.io/crates/v/mockem.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/mockem)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-mockem-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/mockem)

Mock any function in Rust.

Make sure to only use this crate for testing purposes, as it will add a lot of overhead to your code.

## Basic Usage

On the function you want to mock, add the `#[mock]` attribute.

```rust
#[cfg_attr(test, mockem::mock)]
fn foo() -> String {
    format!("foo")
}

fn bar() -> String {
    format!("Hello, {}!", foo())
}

#[test]
fn test_fn() {
    use mockem::MockCall;
    
    foo.mock_ret("mockem".to_owned());
    foo.mock_continue();
    foo.mock_ret("mockem2".to_owned());
    
    assert_eq!(&bar(), "Hello, mockem!");
    assert_eq!(&bar(), "Hello, foo!");
    assert_eq!(&bar(), "Hello, mockem2!");
    assert_eq!(&bar(), "Hello, foo!");
}
```

## Impl Blocks

If you want to mock impl methods, add the `#[mock]` attribute to the impl block.
Do the same for impl trait methods.

This will mock all methods in the impl block.

```rust
struct Foo;

#[cfg_attr(test, mockem::mock)]
impl Foo {
    fn foo(&self) -> String {
        format!("foo")
    }
}

trait Baz {
    fn baz(&self) -> String;
}

#[cfg_attr(test, mockem::mock)]
impl Baz for Foo {
    fn baz(&self) -> String {
        format!("baz")
    }
}

fn bar() -> String {
    format!("Hello, {} and {}!", Foo.foo(), Foo.baz())
}

#[test]
fn test_fn() {
    use mockem::MockCall;
    
    Foo::foo.mock_ret("mockem".to_owned());
    Foo::baz.mock_ret("mockem2".to_owned());
    
    assert_eq!(&bar(), "Hello, mockem and mockem2!");
}
```

## Async Functions

Async functions are also supported.

```rust
use async_trait::async_trait;

struct Foo;

#[cfg_attr(test, mockem::mock)]
impl Foo {
    async fn foo(&self) -> String {
        format!("foo")
    }
}

#[async_trait]
trait Baz {
    async fn baz(&self) -> String;
}

// also works with async_trait
// but you must place #[mock] above #[async_trait]
#[cfg_attr(test, mockem::mock)]
#[async_trait]
impl Baz for Foo {
    async fn baz(&self) -> String {
        format!("baz")
    }
}

async fn bar() -> String {
    format!("Hello, {} and {}!", Foo.foo().await, Foo.baz().await)
}

#[test]
fn test_fn() {
    use mockem::MockCall;
    
    Foo::foo.mock_ret("mockem".to_owned());
    Foo::baz.mock_ret("mockem2".to_owned());
    
    assert_eq!(&bar().await, "Hello, mockem and mockem2!");
}
```

