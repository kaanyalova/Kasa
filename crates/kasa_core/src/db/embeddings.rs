use anyhow::{Ok, Result};
use sqlx::{Pool, QueryBuilder, Sqlite};

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
        QueryBuilder::new("INSERT INTO MediaEmbeddingMeta (hash)");

    metadata_builder.push_values(&results, |mut b, record| {
        b.push_bind(&record.hash);
    });

    metadata_builder.push(" RETURNING id");

    let ids: Vec<i64> = metadata_builder
        .build_query_scalar()
        .fetch_all(pool)
        .await?;

    let mut embeddings_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("INSERT INTO MediaEmbedding (id, embedding) ");

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
