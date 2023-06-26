fn main() {}

#[test]
fn test_srlz() {
    use bevy::prelude::AppTypeRegistry;
    use bevy::scene::serde::SceneDeserializer;
    use canvas_test::engine::Engine;
    use serde::de::DeserializeSeed;

    let mut engie = Engine::new();
    engie.connect_local();
    engie.test_tick();

    let stuffing = engie.serialize_world();

    println!("{}", String::from_utf8(stuffing.clone()).unwrap());

    println!("{:?}", stuffing.len());

    let scene = {
        // let srlz = String::from_utf8(scene).unwrap();
        // web_sys::console::log_1(&srlz.clone().into());
        let mut deserializer = ron::de::Deserializer::from_bytes(&stuffing).unwrap();
        let app = engie.get_app();
        let app = app.lock().unwrap();
        let scene_deserializer = SceneDeserializer {
            type_registry: &app.world.resource::<AppTypeRegistry>().read(),
        };
        scene_deserializer.deserialize(&mut deserializer).unwrap()
    };
}
