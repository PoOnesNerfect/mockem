use async_trait::async_trait;
use mockem::{mock, MockCall};

struct Foo;

#[async_trait]
trait Bar {
    async fn bar(&self) -> String;
}

#[mock]
#[async_trait]
impl Bar for Foo {
    async fn bar(&self) -> String {
        format!("bar")
    }
}

async fn baz() -> String {
    format!("Hello, {}!", <Foo as Bar>::bar(&Foo).await)
}

#[tokio::test]
async fn test_async_trait() {
    <Foo as Bar>::bar.mock_once(|_| "mockem".to_owned());

    assert_eq!(&baz().await, "Hello, mockem!");
}
