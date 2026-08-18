use log::error;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, QueryBuilder, Sqlite, query_as};
use strum::Display;
use utoipa::{IntoParams, ToSchema};

use crate::db::migrations::init_sqlite_vec0;
#[allow(unused)]
use crate::{
    db::schema::Media,
    test_util::db_utils::{_insert_media_row, insert_hash_tag_pair_row},
};

use super::parse_tags;

pub fn parse() {
    todo!()
}

// Tags separated by commas
// `tag1, tag2`
//
// OR queries
// `tag1 or tag2, tag3, tag4`
//
// NOT queries,
// `tag1, tag2, not tag3` or `tag1, tag2, -tag3`
//
// ORDERING
// `tag1, tag2, order by reverse date`
//
// TIME queries
// `tag1, imported between 3 days ago and now`
// `tag1, imported yesterday`

/// Placeholder search until I implement proper search parsing
/// Only supports searching for Media that have the tags

#[derive(Debug, PartialEq, Default, specta::Type, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct DateRange {
    start: u64,
    end: u64,
}

#[derive(Debug, PartialEq, Default, specta::Type, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SearchCriteria {
    contains_tags: Vec<String>,
    contains_tags_or_group: Vec<Vec<String>>,
    excludes_tags: Vec<String>,
    order_by_date: DateOrderCriteria,
    order_by_resolution: ResolutionOrderCriteria,
    date_range: Option<DateRange>,
    favorites_only: bool,
}

#[derive(Debug, PartialEq, Default, specta::Type, Serialize, Deserialize, Display, ToSchema)]
enum DateOrderCriteria {
    #[default]
    NewestFirst,
    OldestFirst,
    None,
}

#[derive(Debug, PartialEq, Default, specta::Type, Serialize, Deserialize, ToSchema)]
enum ResolutionOrderCriteria {
    #[default]
    None,
    HighestFirst,
    LowestFirst,
}

impl SearchCriteria {
    pub fn parse_from_str(input: &str) -> Self {
        let mut contains_tags = vec![];
        let mut contains_tags_or_group = vec![];
        let mut excludes_tags = vec![];
        let mut order_by_criteria_date: DateOrderCriteria = DateOrderCriteria::None;
        let mut order_by_criteria_resolution = ResolutionOrderCriteria::None;

        let mut favorites_only = false;

        let or_separator_regex = Regex::new(r#"(?i)\|| or "#).unwrap();

        // split the input at the commas
        let separated_by_commas: Vec<&str> = input.split(',').collect();

        for token in separated_by_commas {
            // trim the whitespace
            // why is this clone necessary, nom errors out otherwise
            let token = token.trim();

            // We don't want to parse empty tokens
            if token.is_empty() {
                continue;
            }

            // an exclude token
            if token.starts_with('-') {
                excludes_tags.push(
                    token
                        .strip_prefix("-")
                        .expect("Token stars with '-', but cannot remove prefix '-'")
                        .to_owned(),
                );
            }
            // token that is separated by "OR" tags, separate it by "or" or "|"

            // case insensitive, matches "or"s surrounded by whitespace, and "|"s
            else if or_separator_regex.is_match(token) {
                let split: Vec<&str> = or_separator_regex.split(token).collect();
                contains_tags_or_group.push(split.iter().map(|i| i.to_string()).collect());
            }
            // order by
            else if token.to_lowercase().contains("order by") {
                let ordering_criteria_string = token.strip_prefix("order by").unwrap().trim();

                let ordering_criteria_date_parsed = match ordering_criteria_string {
                    // sort by date in order
                    "date" => DateOrderCriteria::NewestFirst,
                    "time" => DateOrderCriteria::NewestFirst,
                    "added" => DateOrderCriteria::NewestFirst,

                    // sort by date in reverse order
                    "date descending" => DateOrderCriteria::OldestFirst,
                    "date reverse" => DateOrderCriteria::OldestFirst,
                    "time descending" => DateOrderCriteria::OldestFirst,
                    "time reverse" => DateOrderCriteria::OldestFirst,
                    "added reverse" => DateOrderCriteria::OldestFirst,
                    "added descending" => DateOrderCriteria::OldestFirst,

                    _ => DateOrderCriteria::None,
                };

                order_by_criteria_resolution = match ordering_criteria_string {
                    "resolution" => ResolutionOrderCriteria::HighestFirst,
                    "resolution reverse" => ResolutionOrderCriteria::LowestFirst,
                    "resolution descending" => ResolutionOrderCriteria::LowestFirst,
                    _ => ResolutionOrderCriteria::None,
                };

                order_by_criteria_date = ordering_criteria_date_parsed;
            }
            // date range
            else if token.starts_with("from") {
                todo!()
            } else if token.to_lowercase() == "favorites" {
                favorites_only = true
            } else {
                contains_tags.push(token.to_string());
            }
            // a regular tag
        }

        SearchCriteria {
            contains_tags,
            contains_tags_or_group,
            excludes_tags,
            order_by_date: order_by_criteria_date,
            order_by_resolution: order_by_criteria_resolution,
            date_range: None,
            favorites_only,
        }
    }

    /*


    -- Example for 1boy, 1girl, general OR sensitive

    SELECT m.*
    FROM Media m, HashTagPair htp
    WHERE m.hash = htp.hash
    AND (htp.tag_name IN ('1girl', '1boy'))
    AND m.hash IN (
        SELECT m.hash
        FROM Media m, HashTagPair htp
        WHERE m.hash = htp.hash
        AND (htp.tag_name IN ('general', 'sensitive'))  -- Fixed typo in 'sensitive'
    )
    GROUP BY m.hash
    HAVING COUNT(m.hash) = 2


         */

    pub fn to_query(&self) -> QueryBuilder<Sqlite> {
        // Handle cases where we are searching for "all tags" (empty query) or "only excludes"
        if self.contains_tags.is_empty() && self.contains_tags_or_group.is_empty() {
            let mut query_builder = QueryBuilder::new("SELECT DISTINCT m.* FROM Media m");

            if self.order_by_resolution != ResolutionOrderCriteria::None {
                query_builder.push(" INNER JOIN Image img ON m.hash = img.hash");
            }

            let mut has_where = false;

            // Handle case where we only want to exclude tags
            if !self.excludes_tags.is_empty() {
                query_builder.push(
                    " WHERE m.hash NOT IN (
                     SELECT htp.hash FROM HashTagPair htp WHERE htp.tag_name IN (",
                );

                let mut separated = query_builder.separated(",");
                for tag in &self.excludes_tags {
                    separated.push_bind(tag);
                }
                query_builder.push("))");
                has_where = true;
            }

            if self.favorites_only {
                if has_where {
                    query_builder.push(" AND m.is_favorite = true");
                } else {
                    query_builder.push(" WHERE m.is_favorite = true");
                }
            }

            self.apply_order_by(&mut query_builder);
            return query_builder;
        }

        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT DISTINCT m.* FROM HashTagPair htp, Media m");

        if self.order_by_resolution != ResolutionOrderCriteria::None {
            query_builder.push(" INNER JOIN Image img ON m.hash = img.hash");
        }

        query_builder.push(" WHERE m.hash = htp.hash ");

        if self.favorites_only {
            query_builder.push("AND m.is_favorite = true");
        }
        // hacky way of only querying for m.hash = htp.hash without any tags being searched
        //if !self.contains_tags.is_empty() && !self.contains_tags_or_group.is_empty() {
        //
        //}

        // add the query for basic "includes tag" search parameter
        if !self.contains_tags.is_empty() {
            query_builder.push("AND htp.tag_name IN (");

            let mut separated = query_builder.separated(", ");

            for tag in &self.contains_tags {
                separated.push_bind(tag);
            }
            separated.push_unseparated(") ");
        }

        for tag_group in &self.contains_tags_or_group {
            query_builder.push(
                "
            AND m.hash IN (
            SELECT m.hash
            FROM Media m, HashTagPair htp
            WHERE m.hash = htp.hash
            AND (htp.tag_name IN (
            ",
            );

            let mut separated = query_builder.separated(", ");
            for tag in tag_group {
                separated.push_bind(tag);
            }
            separated.push_unseparated(") ");

            query_builder.push(")) ");
        }

        // Exclude tags
        if !self.excludes_tags.is_empty() {
            query_builder.push(
                "
                AND m.hash NOT IN (
                SELECT m.hash
                FROM Media m, HashTagPair htp
                WHERE m.hash = htp.hash
                AND (htp.tag_name IN (
            ",
            );

            let mut separated = query_builder.separated(",");

            for tag in &self.excludes_tags {
                separated.push_bind(tag);
            }

            separated.push_unseparated(")");

            query_builder.push("))");
        }

        query_builder.push("GROUP BY m.hash");

        if !self.contains_tags.is_empty() {
            query_builder.push(
                "
        HAVING COUNT(m.hash) =
        ",
            );
            query_builder.push_bind(self.contains_tags.len() as i64);
        }

        self.apply_order_by(&mut query_builder);

        query_builder

        /*
        let contains_tags = &self
            .contains_tags
            .iter()
            .filter(|t| matches!(t, SearchContainsType::ContainsTag));

        for tag in contains_tags {}

        for contains in &self.contains_tags {
            match contains {
                SearchContainsType::ContainsTag(tag) => {
                    query_builder.push("AND (htp.tag_name IN (");
                    let mut separated = query_builder.separated(", ");
                }
                SearchContainsType::ContainTagsOr(tags) => todo!(),
            }
        }
        */
    }

    // Add this method to implement the ordering functionality
    fn apply_order_by(&self, query_builder: &mut QueryBuilder<Sqlite>) {
        let mut order_parts = vec![];

        match self.order_by_resolution {
            ResolutionOrderCriteria::HighestFirst => order_parts.push("img.pixels DESC"),
            ResolutionOrderCriteria::LowestFirst => order_parts.push("img.pixels ASC"),
            ResolutionOrderCriteria::None => {}
        }

        match self.order_by_date {
            DateOrderCriteria::NewestFirst => order_parts.push("m.time_added DESC"),
            DateOrderCriteria::OldestFirst => order_parts.push("m.time_added ASC"),
            DateOrderCriteria::None => {}
        };

        if !order_parts.is_empty() {
            query_builder.push(" ORDER BY ");
            query_builder.push(order_parts.join(", "));
        }
    }

    pub fn merge(&mut self, other: &Self) {
        self.contains_tags.append(&mut other.contains_tags.clone());
        self.contains_tags_or_group
            .append(&mut other.contains_tags_or_group.clone());
        self.excludes_tags.append(&mut other.excludes_tags.clone());

        self.favorites_only |= other.favorites_only;

        // ordering is not merged as it is a single value and should always prioritize the searchbar value
    }
}

/*
figure out a way of loading extensions
#[sqlx::test]
async fn test_sql_query_gen(pool: Pool<Sqlite>) {
    use sqlx::migrate;

    unsafe {
        init_sqlite_vec0();
    }

    migrate!("../../migrations/db").run(&pool).await.unwrap();

    let q = SearchCriteria::parse_from_str("foo, bar, python OR javascript, -csharp");
    let mut q = q.to_query();

    let media1 = Media {
        hash: "123".to_string(),
        media_type: "Image".to_string(),
        thumb_path: Some("nowhere".to_string()),
        thumbnail_x: 123,
        thumbnail_y: 123,
        filesize: 9999,
        mime: None,
        time_added: 0,
        has_file_ref: true,
        hide: false,
        is_valid: true,
        is_favorite: false,
    };

    let media2 = Media {
        hash: "124".to_string(),
        media_type: "Image".to_string(),
        thumb_path: Some("nowhere".to_string()),
        thumbnail_x: 123,
        thumbnail_y: 123,
        filesize: 9999,
        mime: None,
        time_added: 0,
        has_file_ref: true,
        hide: false,
        is_valid: true,
        is_favorite: false,
    };

    let media3 = Media {
        hash: "125".to_string(),
        media_type: "Image".to_string(),
        thumb_path: Some("nowhere".to_string()),
        thumbnail_x: 123,
        thumbnail_y: 123,
        filesize: 9999,
        mime: None,
        time_added: 0,
        has_file_ref: true,
        hide: false,
        is_valid: true,
        is_favorite: false,
    };

    let media4 = Media {
        hash: "126".to_string(),
        media_type: "Image".to_string(),
        thumb_path: Some("nowhere".to_string()),
        thumbnail_x: 123,
        thumbnail_y: 123,
        filesize: 9999,
        mime: None,
        time_added: 0,
        has_file_ref: true,
        hide: false,
        is_valid: true,
        is_favorite: false,
    };

    _insert_media_row(&pool, &media1).await;
    _insert_media_row(&pool, &media2).await;
    _insert_media_row(&pool, &media3).await;
    _insert_media_row(&pool, &media4).await;

    insert_hash_tag_pair_row("123", "foo", &pool).await;
    insert_hash_tag_pair_row("123", "bar", &pool).await;
    insert_hash_tag_pair_row("123", "python", &pool).await;

    insert_hash_tag_pair_row("124", "foo", &pool).await;
    insert_hash_tag_pair_row("124", "bar", &pool).await;
    insert_hash_tag_pair_row("124", "javascript", &pool).await;

    insert_hash_tag_pair_row("125", "foo", &pool).await;
    insert_hash_tag_pair_row("125", "bar", &pool).await;
    insert_hash_tag_pair_row("125", "rust", &pool).await;

    insert_hash_tag_pair_row("126", "foo", &pool).await;
    insert_hash_tag_pair_row("126", "bar", &pool).await;
    insert_hash_tag_pair_row("126", "python", &pool).await;
    insert_hash_tag_pair_row("126", "csharp", &pool).await;

    let queried_media: Vec<Media> = q.build_query_as().fetch_all(&pool).await.unwrap();

    assert!(queried_media.contains(&media1));
    assert!(queried_media.contains(&media2));
    assert!(!queried_media.contains(&media3));
    assert!(!queried_media.contains(&media4))
}

/*
#[test]
fn test_search_parsing() {
    let input_string = "foo, bar, python OR javascript, -rust, order by date";

    let search_criteria = SearchCriteria::parse_from_str(&input_string);

    let expected = SearchCriteria {
        contains_tags: vec![
            SearchContainsType::ContainsTag("foo".to_owned()),
            SearchContainsType::ContainsTag("bar".to_owned()),
            SearchContainsType::ContainTagsOr(vec!["python".to_owned(), "javascript".to_owned()]),
        ],
        excludes_tags: vec!["rust".to_owned()],
        order_by: vec![OrderCriteria::NewestFirst],
    };
    assert_eq!(search_criteria, expected);
}

*/
pub async fn search_impl() {
    // http://web.archive.org/web/20150813211028/http://tagging.pui.ch/post/37027745720/tags-database-schemas
}

pub async fn search_simple_impl(raw_input: &str, pool: &Pool<Sqlite>) -> Vec<Media> {
    let tags = parse_tags(raw_input);

    // show all Media on empty search
    if tags.is_empty() {
        return query_as("SELECT * FROM Media")
            .fetch_all(pool)
            .await
            .unwrap();
    }

    let mut query_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("SELECT m.* FROM HashTagPair htp, Media m WHERE (htp.tag_name) IN (");

    let mut separated = query_builder.separated(", ");

    for tag in &tags {
        separated.push_bind(tag);
    }

    separated.push_unseparated(") ");

    query_builder.push("AND m.hash = htp.hash GROUP BY m.hash");

    query_builder.push("HAVING COUNT (m.hash) = ");
    query_builder.push_bind(tags.len() as i32);

    query_builder.push("AND m.has_file_ref = true");

    let query = query_builder.build_query_as::<Media>();

    query.fetch_all(pool).await.unwrap()
}
*/
