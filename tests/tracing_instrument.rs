use async_trait::async_trait;
use mockem::{mock, MockCall};
use tracing::instrument;

struct Foo;

#[async_trait]
trait Bar {
    async fn bar(&self, b: &str) -> String;
}

#[mock]
#[async_trait]
impl Bar for Foo {
    #[instrument(skip(self))]
    async fn bar(&self, b: &str) -> String {
        format!("bar")
    }
}

async fn baz(b: &str) -> String {
    format!("Hello, {}!", <Foo as Bar>::bar(&Foo, b).await)
}

#[tokio::test]
async fn test_async_trait() {
    <Foo as Bar>::bar.mock_ret("mockem".to_owned());

    assert_eq!(&baz("baz").await, "Hello, mockem!");
}
