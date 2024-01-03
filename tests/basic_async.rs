use mockem::{mock, MockCall};

#[mock]
async fn foo() -> String {
    format!("foo")
}

struct Foo;

impl Foo {
    async fn bar(&self) -> String {
        format!("Hello, {}!", foo().await)
    }
}

#[tokio::test]
async fn test_async() {
    foo.mock_ret("mockem".to_owned());

    assert_eq!(&Foo.bar().await, "Hello, mockem!");
}
