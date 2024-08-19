use std::collections::HashMap;

use sqlx::{sqlite::SqlitePoolOptions, QueryBuilder, Sqlite};

use crate::PopulateTagsArgs;

struct Val {
    name: String,
    count: i64,
}

pub async fn populate_tags(args: PopulateTagsArgs) {
    const BIND_LIMIT: usize = 65535;

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(args.db_path.to_str().unwrap())
        .await
        .unwrap();

    let json_file = tokio::fs::read(args.tags_path).await.unwrap();

    let tags_map: HashMap<String, i64> = serde_json::from_slice(&json_file).unwrap();

    let tags: Vec<Val> = tags_map
        .into_iter()
        .map(|(k, v)| Val { name: k, count: v })
        .collect();

    let mut query_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("INSERT INTO Tag (name,count) ");

    for chunk in tags.chunks(BIND_LIMIT / 4) {
        query_builder.push_values(chunk.iter().take(BIND_LIMIT / 4), |mut b, tag| {
            b.push_bind(&tag.name).push_bind(&tag.count); // Remove clone ?
        });
        let query = query_builder.build();

        query.execute(&pool).await.unwrap();

        query_builder.reset();
    }

    /*
    for entry in tags {
        query("INSERT INTO Tag(name, count) VALUES (?, ?)")
            .bind(entry.count)
            .execute(&pool)
            .await
            .unwrap();
    }
     */
}
