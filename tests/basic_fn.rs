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
    let mut c = 4;
    {
        foo.mock_repeat(None, move || {
            if c > 0 {
                c -= 1;
                println!("c: {c}");
            }
            "mockem".to_owned()
        });
    }

    println!("here c: {c}");

    assert_eq!(&Foo.bar(), "Hello, mockem!");
    assert_eq!(&Foo.bar(), "Hello, mockem!");
    assert_eq!(&Foo.bar(), "Hello, mockem!");
    assert_eq!(&Foo.bar(), "Hello, mockem!");
    assert_eq!(&Foo.bar(), "Hello, mockem!");
    assert_eq!(&Foo.bar(), "Hello, mockem!");
    assert_eq!(&Foo.bar(), "Hello, mockem!");

    println!("here c: {c}");

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
    trim.mock_once(|a| a.trim_start_matches("s"));

    let b = "bar";
    trim.mock_once(|_| b);

    assert_eq!(trim("star"), "tar");
    assert_eq!(trim("star"), "bar");
}
