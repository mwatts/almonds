mod shared;
mod workspaces;

use std::sync::Arc;

use almond_kernel::{
    adapters::{
        bookmarks::{BookmarkTag, CreateBookmark},
        meta::RequestMeta,
        workspace::CreateWorkspace,
    },
    error::KernelError,
    repositories::{
        bookmarks::BookmarkRepository,
        prelude::{BookmarkRepositoryExt, WorkspaceRepositoryExt},
    },
};

use fake::{
    faker::{
        company::en::Industry,
        internet::en::IPv6,
        lorem::en::{Paragraph, Word},
    },
    Fake,
};

use shared::*;
use workspaces::*;

use tokio::sync::OnceCell;
use uuid::Uuid;

static BOOKMARK_REPO: OnceCell<BookmarkRepository> = OnceCell::const_new();

pub async fn get_bookmark_repository() -> &'static BookmarkRepository {
    BOOKMARK_REPO
        .get_or_init(|| async {
            let db = get_db().await;
            BookmarkRepository::new(Arc::new(db.to_owned()))
        })
        .await
}

#[tokio::test]
async fn test_create_without_workspace_bookmarks() -> Result<(), KernelError> {
    let repo = get_bookmark_repository().await;

    let payload = CreateBookmark {
        title: Industry().fake(),
        url: IPv6().fake(),
        tag: BookmarkTag::Design,
    };

    let meta = RequestMeta {
        workspace_identifier: Uuid::new_v4(),
    };

    let create_resp = repo.create(&payload, Some(meta)).await;

    assert!(create_resp.is_err());

    Ok(())
}

#[tokio::test]
async fn test_create_with_workspace_bookmarks() -> Result<(), KernelError> {
    let repo = get_bookmark_repository().await;
    let workspace_repo = get_workspace_repository().await;

    let test_workspace = workspace_repo
        .create_workspace(CreateWorkspace {
            name: Word().fake(),
            description: Paragraph(1..2).fake(),
        })
        .await?;

    let payload = CreateBookmark {
        title: Industry().fake(),
        url: IPv6().fake(),
        tag: BookmarkTag::Design,
    };

    let meta = RequestMeta {
        workspace_identifier: test_workspace.identifier,
    };

    let create_resp = repo.create(&payload, Some(meta)).await?;

    assert_eq!(create_resp.title, payload.title);
    assert_eq!(create_resp.workspace_identifier, Some(test_workspace.identifier));

    Ok(())
}