use mockem::{mock, MockCall};

struct Foo;

trait Bar {
    fn bar(&self) -> String;
}

#[mock]
impl Bar for Foo {
    fn bar(&self) -> String {
        format!("bar")
    }
}

fn baz() -> String {
    format!("Hello, {}!", <Foo as Bar>::bar(&Foo))
}

#[test]
fn test_trait() {
    <Foo as Bar>::bar.mock_ret(|_| "mockem".to_owned());

    assert_eq!(&baz(), "Hello, mockem!");
}
