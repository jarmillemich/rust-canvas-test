fn main() {}

#[test]
fn test_serialization() {
    use bevy::prelude::AppTypeRegistry;
    use bevy::scene::serde::SceneDeserializer;
    use canvas_test::core::networking::serialize_world;
    use canvas_test::engine::Engine;
    use serde::de::DeserializeSeed;

    let mut engine = Engine::new();
    engine.connect_local();
    engine.test_tick();

    let stuffing = serialize_world(&engine.get_app().lock().unwrap().world);

    println!("{}", String::from_utf8(stuffing.clone()).unwrap());

    println!("{:?}", stuffing.len());

    let _scene = {
        let mut deserializer = ron::de::Deserializer::from_bytes(&stuffing).unwrap();
        let app = engine.get_app();
        let app = app.lock().unwrap();
        let scene_deserializer = SceneDeserializer {
            type_registry: &app.world.resource::<AppTypeRegistry>().read(),
        };
        scene_deserializer.deserialize(&mut deserializer).unwrap()
    };
}
