use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use warp::Rejection;

use serde_json::json;

use crate::auth::Claims;
use crate::scraper::{mangasee::Mangasee, repository, GetParams, Params, Scraping};


pub async fn list_sources(db: PgPool) -> Result<impl warp::Reply, Rejection> {
    let sources = sqlx::query!("SELECT name FROM source").fetch_all(&db).await;

    let sources = sources.unwrap().iter().map(|source| source.name.clone()).collect::<Vec<String>>();

    Ok(warp::reply::json(&json!(
        {
            "sources": sources,
            "status": "success"
        }
    )))
}

pub async fn list_mangas(
    source: String,
    param: Params,
    db: PgPool,
) -> Result<impl warp::Reply, Rejection> {
    if let Ok(url) = repository::get_source_url(source.clone(), db.clone()).await {
        let mangas = Mangasee::get_mangas(&url, param);

        for m in mangas.clone().mangas {
            sqlx::query!(
                "INSERT INTO manga(
                    source_id,
                    title,
                    path,
                    thumbnail_url
                    ) VALUES (
                    (SELECT id FROM source WHERE name = $1),
                    $2,
                    $3,
                    $4) ON CONFLICT DO NOTHING",
                source, m.title, m.path, m.thumbnail_url,
            ).execute(&db).await;
        }
        return Ok(warp::reply::json(&mangas));
    }
    Err(warp::reject())
}

pub async fn get_manga_info(
    source: String,
    title: String,
    claim: Claims,
    db: PgPool,
) -> Result<impl warp::Reply, Rejection> {
    let title = decode_title(title);
    if let Ok(manga) =
        repository::get_manga_detail(source.clone(), title.clone(), claim.sub.clone(), db.clone()).await
    {
        return Ok(warp::reply::json(&manga));
    } else if let Ok(url) = repository::get_manga_url(source.clone(), title.clone(), db.clone()).await {
        let manga = Mangasee::get_manga_info(&url);

        sqlx::query!(
            "UPDATE manga SET author = $1, status = $2, description = $3
                WHERE manga.source_id = (
                SELECT source.id FROM source
                WHERE source.name = $4)
                AND manga.title = $5",
                manga.manga.author,
                manga.manga.status,
                manga.manga.description,
                source,
                title,
        ).execute(&db).await;

        return Ok(warp::reply::json(&manga));
    }
    Err(warp::reject())
}

pub async fn get_chapters(
    source: String,
    title: String,
    claim: Claims,
    param: GetParams,
    db: PgPool,
) -> Result<impl warp::Reply, Rejection> {
    let title = decode_title(title);
    if !param.refresh.unwrap_or(false) {
        match repository::get_chapters(source.clone(), title.clone(), claim.sub, db.clone()).await {
            Ok(chapter) => return Ok(warp::reply::json(&chapter)),
            Err(e) => {}
        };
    }

    if let Ok(url) = repository::get_manga_url(source.clone(), title.clone(), db.clone()).await {
        let chapter = Mangasee::get_chapters(&url);

        for c in chapter.clone().chapters {
            sqlx::query!(
                "INSERT INTO chapter(manga_id, number, path, uploaded)
                VALUES(
                (SELECT manga.id FROM manga
                JOIN source ON source.id = manga.source_id
                WHERE source.name = $1 AND title = $2 ),
                $3,
                $4,
                $5) ON CONFLICT DO NOTHING",
                source, title, c.no, c.url, c.uploaded,
            ).execute(&db).await;
        }
        return Ok(warp::reply::json(&chapter));
    }
    Err(warp::reject())
}

pub async fn get_pages(
    source: String,
    title: String,
    chapter: String,
    param: GetParams,
    db: PgPool,
) -> Result<impl warp::Reply, Rejection> {
    let title = decode_title(title);
    if let Ok(url) =
        repository::get_chapter_url(source.clone(), title.clone(), chapter.clone(), db.clone()).await
    {
        let pages = Mangasee::get_pages(&url);
        for i in 0..pages.pages.len() {
            sqlx::query!(
                "INSERT INTO page(chapter_id, rank, url)
                VALUES(
                (SELECT chapter.id FROM chapter
                JOIN manga ON manga.id = chapter.manga_id
                JOIN source ON source.id = manga.source_id
                WHERE source.name = $1 AND manga.title = $2 AND chapter.number = $3),
                $4,
                $5) ON CONFLICT DO NOTHING",
                source, title, chapter, (i as i32), pages.pages[i],
            )
            .execute(&db).await;
        }
        return Ok(warp::reply::json(&pages));
    }
    Err(warp::reject())
}

fn encode_title(title: String) -> String {
    base64::encode_config(&title, base64::URL_SAFE_NO_PAD)
}

fn decode_title(encoded: String) -> String {
    String::from_utf8(base64::decode_config(encoded, base64::URL_SAFE_NO_PAD).unwrap()).unwrap()
}