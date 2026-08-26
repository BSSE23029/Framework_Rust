use super::*;

pub(super) async fn handle_key(app: &mut App, code: KeyCode, _mods: KeyModifiers) {
    if modal_basic::handle(app, code).await
        || modal_accounts::handle(app, code).await
        || modal_idp::handle(app, code).await
        || modal_rbac::handle(app, code).await
        || modal_platform::handle(app, code).await
        || modal_special::handle(app, code).await
    {
        return;
    }

    match app.focus {
        Focus::Sidebar => handle_sidebar_key(app, code).await,
        Focus::Content => handle_content_key(app, code).await,
    }
}
