use std::collections::HashMap;

use log::error;
use nom::{
    branch::alt,
    bytes::{
        complete::{tag_no_case, take_till, take_until},
        streaming::tag,
    },
    multi::{many0, separated_list0},
};
use regex::Regex;
use ruffle_render_wgpu::wgpu::hal::auxil::db;
use sqlx::{Execute, Pool, QueryBuilder, Sqlite, migrate, query_as, query_builder};

#[allow(unused)]
use crate::{
    db::schema::Media,
    test_util::db_utils::{_insert_media_row, insert_hash_tag_pair_row},
};

use super::tags::parse_tags;

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

#[derive(Debug, PartialEq)]
pub struct SearchCriteria {
    contains_tags: Vec<String>,
    contains_tags_or_group: Vec<Vec<String>>,
    excludes_tags: Vec<String>,
    order_by: OrderCriteria,
}

#[derive(Debug, PartialEq)]
enum OrderCriteria {
    NewestFirst,
    OldestFirst,
    None,
}

impl SearchCriteria {
    pub fn parse_from_str(input: &str) -> Self {
        let mut contains_tags = vec![];
        let mut contains_tags_or_group = vec![];
        let mut excludes_tags = vec![];
        let mut order_by_criteria: Option<OrderCriteria> = None;

        let or_separator_regex = Regex::new(r#"(?i)\|| or "#).unwrap();

        // split the input at the commas
        let separated_by_commas: Vec<&str> = input.split(',').collect();

        for token in separated_by_commas {
            // trim the whitespace
            // why is this clone necessary, nom errors out otherwise
            let token = token.trim();

            // We don't want to parse empty tokens
            if token == "" {
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
                let split: Vec<&str> = or_separator_regex.split(&token).collect();
                contains_tags_or_group.push(split.iter().map(|i| i.to_string()).collect());
            }
            // order by
            else if token.to_lowercase().contains("order by") {
                let ordering_criteria_date_string = token.strip_prefix("order by").unwrap().trim();

                let ordering_criteria_date_parsed = match ordering_criteria_date_string {
                    // sort by date in order
                    "date" => OrderCriteria::NewestFirst,
                    "time" => OrderCriteria::NewestFirst,
                    "added" => OrderCriteria::NewestFirst,

                    // sort by date in reverse order
                    "date descending" => OrderCriteria::OldestFirst,
                    "date reverse" => OrderCriteria::OldestFirst,
                    "time descending" => OrderCriteria::OldestFirst,
                    "time reverse" => OrderCriteria::OldestFirst,
                    "added reverse" => OrderCriteria::OldestFirst,
                    "added descending" => OrderCriteria::OldestFirst,
                    _ => {
                        error!("Invalid order criteria entered on the search box");
                        OrderCriteria::None
                    }
                };

                order_by_criteria = Some(ordering_criteria_date_parsed);
            } else {
                contains_tags.push(token.to_string());
            }
            // a regular tag
        }

        SearchCriteria {
            contains_tags,
            contains_tags_or_group,
            excludes_tags,
            order_by: order_by_criteria.unwrap_or(OrderCriteria::OldestFirst),
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
        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "
            SELECT m.* FROM HashTagPair htp, Media m WHERE
            ",
        );

        // hacky way of only querying for m.hash = htp.hash without any tags being searched
        if self.contains_tags.is_empty() && self.contains_tags_or_group.is_empty() {
            query_builder.push("1 = 1 ");
        } else {
            query_builder.push("m.hash = htp.hash ");
        }

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

        match self.order_by {
            OrderCriteria::NewestFirst => query_builder.push(" ORDER BY m.time_added DESC"),
            OrderCriteria::OldestFirst => query_builder.push(" ORDER BY m.time_added ASC"),
            OrderCriteria::None => query_builder.push(" "),
        };

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
}

#[sqlx::test]
async fn test_sql_query_gen(pool: Pool<Sqlite>) {
    migrate!("../migrations/db").run(&pool).await.unwrap();

    let mut q = SearchCriteria::parse_from_str("foo, bar, python OR javascript, -csharp");
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
    if tags.len() == 0 {
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
