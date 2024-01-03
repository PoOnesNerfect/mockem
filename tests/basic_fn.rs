use mockem::{mock, MockCall};

#[mock]
fn foo() -> String {
    format!("foo")
}

struct Foo;

impl Foo {
    fn bar(&self) -> String {
        format!("Hello, {}!", foo())
    }
}

#[test]
fn test_fn() {
    foo.mock_ret("mockem".to_owned());

    assert_eq!(&Foo.bar(), "Hello, mockem!");
}
