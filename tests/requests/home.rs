use bookclub::app::App;
use loco_rs::testing;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn can_request_root() {
    testing::request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/home").await;
        assert_eq!(res.status_code(), 301);
    })
    .await;
}
