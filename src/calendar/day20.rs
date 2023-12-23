use super::AppError;
use axum::{body::Bytes, http::StatusCode, routing::post, Router};
use git2::{Commit, Object, Repository};
use std::io::Cursor;
use tar::Archive;
use tempfile::tempdir;

pub fn router() -> Router {
    let regular = Router::new()
        .route("/archive_files", post(count_files_in_archive))
        .route("/archive_files_size", post(file_sizes_sum));
    let bonus = Router::new().route("/cookie", post(find_commit_author));
    Router::new().nest("/20", regular).nest("/20", bonus)
}

async fn count_files_in_archive(payload: Bytes) -> Result<String, AppError> {
    let cursor = Cursor::new(payload.to_vec());
    let mut archive = Archive::new(cursor);
    Ok(archive.entries().unwrap().count().to_string())
}

async fn file_sizes_sum(payload: Bytes) -> Result<String, AppError> {
    let cursor = Cursor::new(payload.to_vec());
    let mut archive = Archive::new(cursor);
    Ok(archive
        .entries()
        .unwrap()
        .map(|f| f.unwrap().size())
        .sum::<u64>()
        .to_string())
}

async fn find_commit_author(payload: Bytes) -> Result<(StatusCode, String), AppError> {
    let cursor = Cursor::new(payload.to_vec());
    let mut archive = Archive::new(cursor);
    let dir = tempdir().unwrap();
    archive.unpack(dir.path()).unwrap();
    let repo = Repository::open(dir.path()).expect("Unable to open repo");
    let commit = repo
        .find_branch("christmas", git2::BranchType::Local)
        .expect("branch does not exist")
        .get()
        .peel_to_commit()
        .expect("could not extract commit");
    let cookie = find_cookie_commit(0, &commit, &repo).1.unwrap();

    dbg!(cookie.author().name().unwrap().to_string());
    Ok((
        StatusCode::OK,
        format!(
            "{} {}",
            cookie.author().name().unwrap_or_default(),
            cookie.id()
        ),
    ))
}

fn find_cookie_commit<'a>(
    count: u32,
    commit: &Commit<'a>,
    repo: &Repository,
) -> (u32, Option<Commit<'a>>) {
    let tree = commit.tree().unwrap();
    let mut commits: Vec<Object> = Vec::new();
    tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
        if entry.name() == Some("santa.txt") {
            commits.push(entry.clone().to_object(repo).unwrap());
        }
        git2::TreeWalkResult::Ok
    })
    .unwrap();
    let strings: Vec<&str> = commits
        .iter()
        .filter_map(|o| o.as_blob())
        .map(git2::Blob::content)
        .flat_map(|c| std::str::from_utf8(c))
        .filter(|s| s.contains("COOKIE"))
        .collect();
    if !strings.is_empty() {
        return (count, Some(commit.clone()));
    }
    commit
        .parents()
        .map(|p| find_cookie_commit(count + 1, &p, repo))
        .min_by_key(|c| c.0)
        .unwrap_or((0, None))
}
