bench_db_insert_tags:
    yes | sqlx database reset 
    time cargo run --release --package kasa_cli_utils populate-tags --tags-path "/home/kaan/Belgeler/0000_Projects/Kasa/kasa_cli_utils/danbooru.json" --db-path "/home/kaan/Belgeler/0000_Projects/Kasa/dev.db"


