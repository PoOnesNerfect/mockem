use mockem::{mock, ClearMocks, MockCall};

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
    fn f() -> String {
        foo.mock_ret(f);
        "mockem".to_owned()
    }
    foo.mock_ret(f);

    assert_eq!(&Foo.bar(), "Hello, mockem!");
    assert_eq!(&Foo.bar(), "Hello, mockem!");
    assert_eq!(&Foo.bar(), "Hello, mockem!");
    assert_eq!(&Foo.bar(), "Hello, mockem!");
    assert_eq!(&Foo.bar(), "Hello, mockem!");
    assert_eq!(&Foo.bar(), "Hello, mockem!");
    assert_eq!(&Foo.bar(), "Hello, mockem!");

    // clears mocks for `foo`
    foo.clear_mocks();

    assert_eq!(&Foo.bar(), "Hello, foo!");
}

#[mock]
fn trim(s: &str) -> &str {
    s.trim()
}

#[test]
fn test_ref() {
    trim.mock_ret(|a| a.trim_start_matches("s"));

    let b = "bar";
    trim.mock_ret(|_| b);

    assert_eq!(trim("star"), "tar");
    assert_eq!(trim("star"), "bar");
}
