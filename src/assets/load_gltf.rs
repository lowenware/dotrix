use std::{
    path::PathBuf,
    sync::{Arc, mpsc, Mutex},
};

use gltf::{
    Gltf,
    buffer::Source,
    animation::util::ReadOutputs,
};

use log::info;

use super::{
    animation::{Animation, Interpolation},
    loader::{Asset, ImportError, Response, load_image},
    mesh::Mesh,
    skin::{Skin, JointId, Joint, JointIndex},
    transform::Transform,
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
            load_node(sender, &name, &node, None, &buffers)?;
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
                    if let Some(stripped) = uri.strip_prefix(URI_BASE64) {
                        base64::decode(stripped)?
                    } else {
                        std::fs::read(path.parent().unwrap().join(uri))?
                    }
                );
            }
        }
    }

    Ok(buffers)
}

fn load_joints(
    joints: &mut Vec<Joint>,
    node: &gltf::Node,
    parent_id: Option<JointId>,
) {
    let local_transform = Transform::from(node.transform());
    let id = node.index();
    joints.push(Joint::new(
        id,
        parent_id,
        node.name().map(String::from),
        local_transform,
    ));

    for child in node.children() {
        load_joints(joints, &child, Some(id));
    }
}

fn load_node(
    sender: &Arc<Mutex<mpsc::Sender<Response>>>,
    name: &str,
    node: &gltf::Node,
    root: Option<&gltf::Node>,
    buffers: &[Vec<u8>],
) -> Result <(), ImportError> {

    if let Some(skin) = node.skin() {
        let reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));
        let inverse_bind_matrices = reader
            .read_inverse_bind_matrices()
            .map(|v| v.map(cgmath::Matrix4::<f32>::from).collect());

        let asset_name = [name, "skin"].join("::");
        let index = skin.joints().map(|j| JointIndex { id: j.index(), inverse_bind_matrix: None}).collect::<Vec<_>>();
        let mut joints: Vec<Joint> = Vec::new();
        load_joints(&mut joints, skin.skeleton().as_ref().or(root).unwrap(), None);

        info!("importing skin as `{}`", asset_name);
        sender.lock().unwrap().send(Response::Skin(
            Asset {
                name: asset_name,
                asset: Skin::new(joints, index, inverse_bind_matrices),
            }
        )).unwrap();
    }

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            load_mesh(sender, name, &primitive, buffers)?;
            let material = primitive.material();
            if let Some(texture) = material.pbr_metallic_roughness().base_color_texture() {
                load_texture(sender, name, &texture, buffers)?;
            }
        }
    }

    let root = root.or(Some(node));
    for child in node.children() {
        let child_name = if let Some(child_name) = child.name() {
            [name, child_name].join("::")
        } else {
            format!("{}.node[{}]", name, child.index())
        };


        load_node(sender, &child_name, &child, root, buffers)?;
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
    let weights = reader.read_weights(0).map(|w| w.into_f32().collect::<Vec<[f32; 4]>>());
    let joints = reader.read_joints(0)       .map(|j| j.into_u16().collect::<Vec<[u16; 4]>>());

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
    gltf_animation: &gltf::Animation,
    buffers: &[Vec<u8>],
) -> Result <(), ImportError> {
    let name = if let Some(animation_name) = gltf_animation.name() {
        [name, animation_name].join("::")
    } else {
        format!("{}.animation[{}]", name, gltf_animation.index())
    };

    info!("importing animation as `{}`", name);

    let mut animation = Animation::new();

    for channel in gltf_animation.channels() {

        let sampler = channel.sampler();
        let interpolation = Interpolation::from(sampler.interpolation());
        let index = channel.target().node().index();
        let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
        let outputs = reader.read_outputs();
        let timestamps = reader.read_inputs().unwrap().collect::<Vec<f32>>();

        match outputs.unwrap() {
            ReadOutputs::Translations(output) => animation.add_translation_channel(
                index, interpolation, timestamps, output.map(cgmath::Vector3::<f32>::from).collect(),
            ),
            ReadOutputs::Rotations(output) => animation.add_rotation_channel(
                index, interpolation, timestamps, output.into_f32()
                    .map(|q| cgmath::Quaternion::<f32>::new(q[3], q[0], q[1], q[2])).collect()
            ),
            ReadOutputs::Scales(output) => animation.add_scale_channel(
                index, interpolation, timestamps, output.map(cgmath::Vector3::<f32>::from).collect()
            ),
            ReadOutputs::MorphTargetWeights(ref _weights) => (),
        };
    }

    sender.lock().unwrap().send(Response::Animation(
        Asset {
            name,
            asset: animation,
        }
    )).unwrap();

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
