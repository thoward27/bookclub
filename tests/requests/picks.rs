use bookclub::app::App;
use loco_rs::testing;
use serial_test::serial;

macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("picks_request");
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn can_get_picks() {
    configure_insta!();

    testing::request::<App, _, _>(|request, ctx| async move {
        testing::seed::<App>(&ctx.db).await.unwrap();

        let res = request.get("/picks").await;
        assert_eq!(res.status_code(), 200);
        print!("{}", res.text());
        assert!(res.text().contains("Atomic Habits"));
    })
    .await;
}
