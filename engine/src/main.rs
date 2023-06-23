use canvas_test::engine::Engine;

fn main() {}

#[test]
fn test_srlz() {
    let mut engie = Engine::new();
    engie.connect_local();
    engie.test_tick();

    let stuffing = engie.serialize_world();

    println!("{:?}", stuffing.len())
}
