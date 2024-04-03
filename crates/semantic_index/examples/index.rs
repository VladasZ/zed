use futures::channel::oneshot;
use gpui::{App, Global, TestAppContext};
use language::language_settings::AllLanguageSettings;
use project::Project;
use semantic_index::SemanticIndex;
use settings::SettingsStore;
use std::path::Path;
use tempfile::tempdir;

pub fn init_test(cx: &mut TestAppContext) {
    _ = cx.update(|cx| {
        let store = SettingsStore::test(cx);
        cx.set_global(store);
        language::init(cx);
        Project::init_settings(cx);
        SettingsStore::update(cx, |store, cx| {
            store.update_user_settings::<AllLanguageSettings>(cx, |_| {});
        });
    });
}

fn main() {
    App::new().run(|cx| {
        let store = SettingsStore::test(cx);
        cx.set_global(store);
        language::init(cx);
        Project::init_settings(cx);
        SettingsStore::update(cx, |store, cx| {
            store.update_user_settings::<AllLanguageSettings>(cx, |_| {});
        });

        cx.spawn(|mut cx| async move {
            let project = Project::example([Path::new("/Users/as-cii/dev/zed")], &mut cx).await;

            let temp_dir = tempdir().unwrap();
            let mut semantic_index = SemanticIndex::new(temp_dir.path()).unwrap();

            let project_index = cx
                .update(|cx| semantic_index.project_index(project.clone(), cx))
                .unwrap();

            let (tx, mut rx) = oneshot::channel();
            let mut tx = Some(tx);
            let subscription = cx.update(|cx| {
                cx.subscribe(&project_index, move |_, event, _| {
                    if let Some(tx) = tx.take() {
                        _ = tx.send(event.clone());
                    }
                })
            });

            let event = rx.await.expect("no event emitted");
            drop(subscription);
            dbg!(event);
        })
        .detach();
    });
}