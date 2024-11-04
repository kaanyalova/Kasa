use sqlx::{query as sqlx_query, Pool, QueryBuilder, Sqlite};

use crate::db::schema::{media_type_to_string, MediaType};

use super::media_types::{DbWritableMediaDataBatch, MediaTypeWithData};

pub async fn write_to_db(
    inputs: DbWritableMediaDataBatch,
    media_type: MediaType,
    pool: &Pool<Sqlite>,
    _pool_thumbs: &Pool<Sqlite>, // TODO delete this unused
) {
    // Write the basic Media data to the db

    // Ignore any duplicate hashes
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        "INSERT OR IGNORE INTO Media(hash, media_type, thumb_path, filesize, mime, time_added, thumbnail_x, thumbnail_y, has_file_ref)",
    );

    query_builder.push_values(inputs.generic_media_data.iter(), |mut b, data| {
        b.push_bind(&data.hash)
            .push_bind(media_type_to_string(&media_type))
            .push_bind(&data.thumb_path)
            .push_bind(data.size as i64)
            .push_bind(&data.mime)
            .push_bind(&data.time_added)
            .push_bind(&data.thumbnail_x)
            .push_bind(&data.thumbnail_y)
            .push_bind(true);
    });

    let query = query_builder.build();

    query.execute(pool).await.unwrap();

    // Write the path info to DB

    let mut query_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("INSERT OR IGNORE INTO Path(hash, path) ");
    query_builder.push_values(inputs.paths.into_iter(), |mut b, data| {
        b.push_bind(data.hash).push_bind(data.path);
    });

    let query = query_builder.build();
    query.execute(pool).await.unwrap();

    // Write specific file metadata

    match media_type {
        MediaType::Image => {
            let mut query_builder: QueryBuilder<Sqlite> =
                QueryBuilder::new("INSERT INTO Image(resolution_x, resolution_y, hash) ");
            // TODO remove clone()
            query_builder.push_values(inputs.media_data.into_iter(), |mut b, data| {
                #[allow(irrefutable_let_patterns)] // what???
                if let MediaTypeWithData::Image(d) = data {
                    b.push_bind(d.resolution_x as i64)
                        .push_bind(d.resolution_y as i64)
                        .push_bind(d.hash);
                }
            });

            let query = query_builder.build();
            query.execute(pool).await.unwrap();
        }
        MediaType::Video => { /* TODO implement video meta */ }
        MediaType::Game => todo!(),
        MediaType::Unknown => todo!(),
    }

    // Mark any unreferenced files
    sqlx_query("UPDATE Media SET has_file_ref = false WHERE hash IN (SELECT Media.hash FROM Path LEFT JOIN Media ON Media.hash = Path.hash WHERE Path.hash IS NULL)").execute(pool).await.unwrap();

    /*
    UPDATE Media
    SET thumb_path = 'Unreferenced'
    WHERE hash NOT IN (
        SELECT Media.hash
        FROM "Path"
        LEFT JOIN Media ON Media.hash = "Path".hash
        WHERE "Path".hash IS NULL
    );

         */

    // Write the thumbnail info to db

    // TODO get them from Local db preferences, or global? idk

    /* *
    let thumbnail_max_x = 256;
    let thumbnail_max_y = 256;

    match media_type {
        MediaType::Image => {
            let mut query_builder: QueryBuilder<Sqlite> =
                QueryBuilder::new("INSERT OR IGNORE INTO Thumbs(hash, x, y, x_max, y_max) ");

            query_builder.push_values(inputs.generic_media_data.into_iter(), |mut b, data| {
                #[allow(irrefutable_let_patterns)] // what???
                b.push_bind(data.hash)
                    .push_bind(data.thumbnail_x)
                    .push_bind(data.thumbnail_y)
                    // TODO add actual values
                    .push_bind(256)
                    .push_bind(256);
            });

            let query = query_builder.build();
            query.execute(pool_thumbs).await.unwrap();
        }
        MediaType::Video => todo!(),
        MediaType::Game => todo!(),
        MediaType::Unknown => todo!(),
    }

    */
}
