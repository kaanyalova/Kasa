use std::collections::VecDeque;

use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, QueryBuilder, Sqlite, prelude::FromRow, query_as};

use crate::index::indexer::index;

pub struct EmbeddingResult {
    pub hash: String,
    pub embedding: Vec<f32>,
}

struct EmbeddingResultWithRowId {
    hash: String,
    embedding: Vec<f32>,
    row_id: i64,
}

pub async fn insert_embeddings(pool: &Pool<Sqlite>, results: Vec<EmbeddingResult>) -> Result<()> {
    let mut metadata_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("INSERT OR REPLACE INTO MediaEmbeddingMeta (hash)");

    metadata_builder.push_values(&results, |mut b, record| {
        b.push_bind(&record.hash);
    });

    metadata_builder.push(" RETURNING id");

    let ids: Vec<i64> = metadata_builder
        .build_query_scalar()
        .fetch_all(pool)
        .await?;

    let mut embeddings_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("INSERT OR REPLACE INTO MediaEmbedding (id, embedding) ");

    if ids.len() == 0 {
        return Ok(());
    }
    let embedding_results_with_row_ids =
        results
            .into_iter()
            .enumerate()
            .map(|(i, result)| EmbeddingResultWithRowId {
                hash: result.hash,
                embedding: result.embedding,
                row_id: ids[i],
            });

    embeddings_builder.push_values(embedding_results_with_row_ids, |mut b, result| {
        b.push_bind(result.row_id);
        let bytes: &[u8] = bytemuck::cast_slice(&result.embedding);
        b.push_bind(bytes);
    });

    embeddings_builder.build().execute(pool).await?;

    Ok(())
}

#[derive(Debug, FromRow, specta::Type, Serialize, Deserialize)]
pub struct EmbeddingDistance {
    pub hash: String,
    pub distance: f32,
}

pub async fn get_top_n_closest_for_media_impl(
    pool: &Pool<Sqlite>,
    hash: &str,
    n: i64,
) -> Result<Vec<EmbeddingDistance>> {
    let mut closest: VecDeque<EmbeddingDistance> = query_as(
        "
        SELECT hash, distance 
        FROM MediaEmbedding me 
        LEFT JOIN MediaEmbeddingMeta mem ON mem.id = me.id 
        WHERE me.embedding MATCH (
            SELECT me.embedding FROM MediaEmbedding me
            JOIN MediaEmbeddingMeta mem ON mem.id = me.id 
            WHERE mem.hash = ?
        )
        AND me.k = ?
        ORDER BY DISTANCE 
        LIMIT ?
    ",
    )
    .bind(hash)
    .bind(n + 1)
    .bind(n + 1)
    .fetch_all(pool)
    .await?
    .into();

    // remove the image itself
    closest.pop_front();

    Ok(closest.into())
}
