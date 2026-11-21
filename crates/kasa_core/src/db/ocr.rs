use std::{borrow::Cow, sync::LazyLock};

use anyhow::Result;
use kasa_ai::TextRegion;
use lindera::{dictionary::load_dictionary, mode::Mode, segmenter::Segmenter};
use sqlx::{Pool, QueryBuilder, Sqlite, types::Json};

pub static JAPANESE_SEGMENTER: LazyLock<Segmenter> = LazyLock::new(|| {
    let dictionary = load_dictionary("embedded://ipadic").unwrap();
    Segmenter::new(Mode::Normal, dictionary, None)
});

pub struct OcrResult {
    pub hash: String,
    pub data: Vec<TextRegion>,
}

pub async fn insert_ocr_results(pool: &Pool<Sqlite>, results: &[OcrResult]) -> Result<()> {
    if results.is_empty() {
        return Ok(());
    }

    let mut builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("INSERT INTO OcrData (hash, regions, text) ");

    builder.push_values(results, |mut separated, result| {
        separated.push_bind(&result.hash);
        separated.push_bind(Json(&result.data));
        separated.push_bind(flatten_ocr_result(result));
    });

    // insert or replace doesn't work on triggers
    builder
        .push(" ON CONFLICT(hash) DO UPDATE SET regions = excluded.regions, text = excluded.text");

    builder.build().execute(pool).await?;

    Ok(())
}

fn flatten_ocr_result(result: &OcrResult) -> String {
    result
        .data
        .iter()
        .filter_map(|region| region.text.as_ref())
        .map(|text| segment_for_fts(text.trim()))
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn segment_for_fts(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let Ok(tokens) = JAPANESE_SEGMENTER.segment(Cow::Borrowed(text)) else {
        return text.to_owned();
    };

    tokens
        .iter()
        .map(|token| token.surface.as_ref())
        .collect::<Vec<_>>()
        .join(" ")
}
