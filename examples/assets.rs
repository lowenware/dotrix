use dotrix::asset;

fn main() {
    let mut assets = asset::Service::new();

    let id = assets.import("female-model.gltf", "female");
    println!("Imported: {:?}", id);

    let id = assets.find("female").unwrap();
    println!("found: {:?}", id);

    let res = assets.get::<asset::Resource>(id).unwrap();
    println!("resource {:?} -> {:?}", res.path(), res.name());

    let id = assets.import("male-model.gltf", "male");
    println!("Imported: {:?}", id);

    let res = assets.get::<asset::Resource>(id).unwrap();
    println!("resource {:?} -> {:?}", res.path(), res.name());

}
