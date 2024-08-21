use kasa_core::db::schema::Image;
use rand::Rng;

pub fn dump_random_gi_layout() {
    let mut images = vec![];
    let width = 1000;

    for _ in 0..=100_000 {
        let x = rand::thread_rng().gen_range(0..2000);
        let y = rand::thread_rng().gen_range(0..2000);

        let data = Image {
            hash: "".to_string(),
            resolution_x: x,
            resolution_y: y,
        };

        images.push(data);
    }

    //let layout = calculate_layout(images, width, 500);

    //let jason = serde_json::to_string(&layout).unwrap();

    //std::fs::write("./dump.json", jason).unwrap();
}
