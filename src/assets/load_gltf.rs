use std::{
    path::PathBuf,
    sync::{ Arc, mpsc, Mutex },
};

use gltf::{
    Gltf,
    buffer::Source,
    animation::util::ReadOutputs,
};

use log::info;

use super::{
    animation::{Animation, JointTransforms},
    mesh::Mesh,
    skin::Skin,
    loader::{ Asset, ImportError, Response, load_image },
};

pub fn load_gltf(
    sender: &Arc<Mutex<mpsc::Sender<Response>>>,
    name: String,
    data: Vec<u8>,
    path: &PathBuf,
) -> Result<(), ImportError>{

    let gltf = Gltf::from_slice(&data)?;
    let buffers = load_buffers(&gltf, path)?;

    for scene in gltf.scenes() {
        for node in scene.nodes() {
            load_node(sender, &name, &node, &buffers)?;
        }
    }

    for animation in gltf.animations() {
        load_animation(sender, &name, &animation, &buffers)?;
    }

    Ok(())
}

fn load_buffers(gltf: &Gltf, path: &PathBuf) -> Result<Vec<Vec<u8>>, ImportError> {
    const URI_BASE64: &str = "data:application/octet-stream;base64,";
    let mut buffers = Vec::new();

    for buffer in gltf.buffers() {
        match buffer.source() {
            Source::Bin => {
                if let Some(blob) = gltf.blob.as_deref() {
                    buffers.push(blob.into());
                } else {
                    return Err(ImportError::Corruption("blob buffer not found"));
                }
            },
            Source::Uri(uri) => {
                buffers.push(
                    if uri.starts_with(URI_BASE64) {
                        base64::decode(&uri[URI_BASE64.len()..])?
                    } else {
                        std::fs::read(path.parent().unwrap().join(uri))?
                    }
                );
            }
        }
    }

    Ok(buffers)
}

fn load_node(
    sender: &Arc<Mutex<mpsc::Sender<Response>>>,
    name: &str,
    node: &gltf::Node,
    buffers: &[Vec<u8>],
) -> Result <(), ImportError> {

    // TODO: decide on if we need transformation as a separate asset?
    // let transform = node.transform();

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            load_mesh(sender, name, &primitive, buffers)?;
            let material = primitive.material();
            if let Some(texture) = material.pbr_metallic_roughness().base_color_texture() {
                load_texture(sender, name, &texture, buffers)?;
            }
        }
    }

    if let Some(skin) = node.skin() {
        let reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));
        if let Some(inverse_bind_matrices) = reader.read_inverse_bind_matrices() {
            let inverse_bind_matrices: Vec<cgmath::Matrix4<f32>> = inverse_bind_matrices.map(
                cgmath::Matrix4::<f32>::from
            ).collect();

            let asset_name = [name, "skin"].join("::");
            sender.lock().unwrap().send(Response::Skin(
                Asset {
                    name: asset_name,
                    asset: Skin::new(inverse_bind_matrices)
                }
            )).unwrap();
        }
    }

    for child in node.children() {
        let child_name = if let Some(child_name) = child.name() {
            [name, child_name].join("::")
        } else {
            format!("{}.node[{}]", name, child.index())
        };

        load_node(sender, &child_name, &child, buffers)?;
    }

    Ok(())
}

fn load_mesh(
    sender: &Arc<Mutex<mpsc::Sender<Response>>>,
    name: &str,
    primitive: &gltf::Primitive,
    buffers: &[Vec<u8>],
) -> Result <(), ImportError> {

    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
    let mode = primitive.mode();

    if mode != gltf::mesh::Mode::Triangles {
        return Err(ImportError::NotImplemented("primitive mode", mode_to_string(mode)));
    }

    let positions = reader.read_positions().map(|p| p.collect::<Vec<[f32; 3]>>());
    let normals = reader.read_normals().map(|n| n.collect::<Vec<[f32; 3]>>());
    let texture = reader.read_tex_coords(0).map(|t| t.into_f32().collect());
    let indices = reader.read_indices().map(|i| i.into_u32().collect::<Vec<u32>>());
    let joints = reader.read_joints(0).map(|v| v.into_u16().collect::<Vec<[u16; 4]>>());
    let weights = reader.read_weights(0).map(|v| v.into_f32().collect::<Vec<[f32; 4]>>());

    let name = [name, "mesh"].join("::");

    info!("importing mesh as `{}`", name);

    sender.lock().unwrap().send(Response::Mesh(
        Asset {
            name,
            asset: Mesh::new(
                positions.unwrap(),
                normals,
                texture,
                indices,
                joints,
                weights,
            )
        }
    )).unwrap();

    Ok(())
}

fn load_texture(
    sender: &Arc<Mutex<mpsc::Sender<Response>>>,
    name: &str,
    texture: &gltf::texture::Info,
    buffers: &[Vec<u8>],
) -> Result <(), ImportError> {

    let source = texture.texture().source().source();
    let name = [name, "texture"].join("::");
    info!("importing texture as `{}`", name);

    let (data, format) = match source {
        gltf::image::Source::Uri { uri, mime_type } => {
            const URI_IMAGE_PNG: &str = "data:image/png;base64,";

            if !uri.starts_with(URI_IMAGE_PNG) {
                return Err(ImportError::NotImplemented("mime type",
                        mime_type.map(String::from)));
            }

            let data = base64::decode(&uri[URI_IMAGE_PNG.len()..])?;
            (data, image::ImageFormat::Png)
        },

        gltf::image::Source::View { view, mime_type } => {
            if mime_type != "image/png" {
                return Err(ImportError::NotImplemented("mime type",
                        Some(String::from(mime_type))));
            }

            let index = view.buffer().index();
            let offset = view.offset();
            let tail = offset + view.length();
            let data = &buffers[index][offset..tail];

            (data.to_vec(), image::ImageFormat::Png)
        }
    };

    load_image(sender, name, data, format)?;

    Ok(())
}

fn load_animation(
    sender: &Arc<Mutex<mpsc::Sender<Response>>>,
    name: &str,
    animation: &gltf::Animation,
    buffers: &[Vec<u8>],
) -> Result <(), ImportError> {
    let name = if let Some(animation_name) = animation.name() {
        [name, animation_name].join("::")
    } else {
        format!("{}.animation[{}]", name, animation.index())
    };

    info!("importing animation as `{}`", name);
    // println!("importing animation as `{}`", name);

    // Vec is used here instead of the HashMap, because we should keep the order of keyframes
    let mut joint_transforms: Vec<(usize, JointTransforms)> = Vec::new();
    let mut keyframes: Option<Vec<f32>> = None;
    for channel in animation.channels() {
        // println!("  Channel: {}", channel.target().node().name().unwrap());
        let node_index = channel.target().node().index();
        let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
        let outputs = reader.read_outputs();

        if keyframes.is_none() {
            keyframes = Some(reader.read_inputs().unwrap().collect::<Vec<f32>>());
        }

        let mut transforms = joint_transforms.iter_mut().find(|(n, _)| n == &node_index);
        if transforms.is_none() {
            joint_transforms.push((node_index, JointTransforms::default()));
            transforms = joint_transforms.last_mut();
        }
        if let Some((_, transform)) = transforms {
            match outputs.unwrap() {
                ReadOutputs::Translations(output) => transform.translations = Some(
                    output.map(cgmath::Vector3::<f32>::from).collect()
                ),
                ReadOutputs::Rotations(output) => transform.rotations = Some(
                    output.into_f32().map(cgmath::Quaternion::<f32>::from).collect()
                ),
                ReadOutputs::Scales(output) => transform.scales = Some(
                    output.map(cgmath::Vector3::<f32>::from).collect()
                ),
                ReadOutputs::MorphTargetWeights(ref _weights) => {},
            };
        }
    }

    if let Some(keyframes) = keyframes {
        let joint_transforms = joint_transforms.into_iter().map(|(_, t)| t).collect();

        sender.lock().unwrap().send(Response::Animation(
            Asset {
                name,
                asset: Animation::new(keyframes, joint_transforms),
            }
        )).unwrap();
    }

    Ok(())
}

fn mode_to_string(mode: gltf::mesh::Mode) -> Option<String> {
    let result = match mode {
        gltf::mesh::Mode::Points => "Points",
        gltf::mesh::Mode::Lines => "Lines",
        gltf::mesh::Mode::LineLoop => "Line Loop",
        gltf::mesh::Mode::LineStrip => "Line Strip",
        gltf::mesh::Mode::Triangles => "Triangles",
        gltf::mesh::Mode::TriangleStrip => "Triangle Strip",
        gltf::mesh::Mode::TriangleFan => "Triangle Fan",
    };
    Some(String::from(result))
}
