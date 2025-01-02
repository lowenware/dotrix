use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use gltf::Gltf;

//use dotrix_assets as assets;
//use dotrix_image as image;
//use dotrix_Image;
//use dotrix_log as log;
//use dotrix_math::{Mat4, Quat, Vec3};
//use dotrix_mesh::animation::{Animation, Interpolation};
//use dotrix_mesh::{Armature, Joint, Mesh};
//use dotrix_pbr::Material;
//use dotrix_types::{vertex, Color};
//use dotrix_types::{Id, Transform};

use crate::log;
use crate::math::{Mat4, Quat, Vec3};
use crate::models::{
    Animation, Armature, Color, Image, ImageFormat, Interpolation, Joint, Material, Mesh,
    Transform3D, VertexJoints, VertexNormal, VertexPosition, VertexTexture, VertexWeights,
};
use crate::utils::Id;

use super::{Asset, ImageLoader, ResourceBundle, ResourceLoader, ResourceTarget};

type JsonIndex = usize;
type ResultIndex = usize;

#[derive(Default)]
struct Output {
    result: Vec<Box<dyn Asset>>,
    loaded_images: HashMap<JsonIndex, ResultIndex>,
    loaded_meshes: HashMap<JsonIndex, ResultIndex>,
    loaded_materials: HashMap<JsonIndex, ResultIndex>,
    loaded_armature: HashMap<JsonIndex, ResultIndex>,
    loaded_joints: HashMap<JsonIndex, Id<Joint>>,
}

/// Gltf file loader
#[derive(Default)]
pub struct GltfLoader;

impl ResourceLoader for GltfLoader {
    fn read(&self, path: &Path, targets: &HashSet<ResourceTarget>) -> ResourceBundle {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(err) => panic!("Could not open GLTF resource file ({path:?}): {err:?}",),
        };
        let metadata = std::fs::metadata(path).expect("Could not read GLTF file metadata");
        let mut data = vec![0; metadata.len() as usize];
        file.read_exact(&mut data)
            .expect("Could not read GLTF resource file into buffer");

        let mut output = Output {
            result: Vec::with_capacity(targets.len()),
            ..Default::default()
        };

        match Gltf::from_slice(&data) {
            Ok(gltf) => {
                if let Some(buffers) = Self::read_buffers(&gltf, path) {
                    let name: String = path
                        .file_stem()
                        .expect("GLTF path has no file name")
                        .to_str()
                        .expect("Could not read file name to string")
                        .into();
                    for scene in gltf.scenes() {
                        for node in scene.nodes() {
                            Self::read_node(&mut output, &node, &buffers, &name, None);
                        }
                    }
                    for animation in gltf.animations() {
                        Self::read_animation(&mut output, &animation, &buffers, &name);
                    }
                }
            }
            Err(err) => log::error!("Could not read GLTF file `{path:?}`: {err:?}"),
        };

        let mut bundle = targets
            .iter()
            .map(|target| (target.clone(), None))
            .collect::<HashMap<_, _>>();

        let no_targets = targets.is_empty();

        for asset in output.result.into_iter() {
            let target = ResourceTarget {
                type_id: asset.type_id(),
                name: asset.name().into(),
            };
            if no_targets || targets.contains(&target) {
                bundle.insert(target, Some(asset));
            }
        }

        ResourceBundle {
            resource: path.into(),
            bundle,
        }
    }
}

impl GltfLoader {
    fn read_buffers(gltf: &Gltf, path: &std::path::Path) -> Option<Vec<Vec<u8>>> {
        const URI_BASE64: &str = "data:application/octet-stream;base64,";
        let mut buffers = Vec::new();

        for buffer in gltf.buffers() {
            match buffer.source() {
                gltf::buffer::Source::Bin => {
                    if let Some(blob) = gltf.blob.as_deref() {
                        buffers.push(blob.into());
                    } else {
                        log::error!("GLTF blob buffer was not found");
                        return None;
                    }
                }
                gltf::buffer::Source::Uri(uri) => {
                    if let Some(stripped) = uri.strip_prefix(URI_BASE64) {
                        match base64_decode(stripped) {
                            Ok(buffer) => buffers.push(buffer),
                            Err(err) => {
                                log::error!("Could not decode Base64 buffer: {err:?}");
                                return None;
                            }
                        };
                    } else {
                        match std::fs::read(path.parent().unwrap().join(uri)) {
                            Ok(buffer) => buffers.push(buffer),
                            Err(err) => {
                                log::error!("Could not read GLTF buffer from file: {err:?}");
                                return None;
                            }
                        };
                    };
                }
            }
        }

        Some(buffers)
    }

    fn read_node(
        output: &mut Output,
        node: &gltf::Node,
        buffers: &[Vec<u8>],
        name: &str,
        root: Option<&gltf::Node>,
    ) {
        if let Some(skin) = node.skin() {
            Self::read_armature(output, &skin, buffers, name, root);
        }

        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                Self::read_mesh(output, &primitive, buffers, name);

                let material = primitive.material();
                Self::read_material(output, &material, buffers, name);
            }
        }

        let root = root.or(Some(node));
        for child in node.children() {
            let child_name = if let Some(child_name) = child.name() {
                [name, child_name].join("::")
            } else {
                format!("{}.node[{}]", name, child.index())
            };

            Self::read_node(output, &child, buffers, &child_name, root);
        }
    }

    fn read_armature(
        output: &mut Output,
        skin: &gltf::Skin,
        buffers: &[Vec<u8>],
        name: &str,
        root: Option<&gltf::Node>,
    ) {
        let skin_index = skin.index();
        if output.loaded_armature.contains_key(&skin_index) {
            return;
        }
        let reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));
        let inverse_bind_matrices = reader.read_inverse_bind_matrices().map(|v| {
            v.map(|mx| Mat4::from_cols_array_2d(&mx))
                .collect::<Vec<_>>()
        });

        let asset_name = [name, "armature"].join("::");

        let index = skin
            .joints()
            .enumerate()
            .map(|(i, node)| {
                (
                    node.name(),
                    inverse_bind_matrices.as_ref().map(|list| &list[i]),
                    node.index(),
                )
            })
            .collect::<Vec<_>>();

        let capacity = index.len();

        let mut joints: HashMap<usize, (Id<Joint>, Joint)> = HashMap::with_capacity(capacity);

        Self::read_joints(
            &mut joints,
            skin.skeleton().as_ref().or(root).unwrap(),
            None,
        );

        let mut armature = Armature::new(asset_name, capacity);
        for (name, inverse_bind_matrix, index) in index.into_iter() {
            let (id, mut joint) = joints.remove(&index).expect("Joint does not exists");
            joint.inverse_bind_matrix = inverse_bind_matrix.cloned();
            armature.add(id, name.map(String::from), joint);
            output.loaded_joints.insert(index, id);
        }
        output
            .loaded_armature
            .insert(skin_index, output.result.len());
        output.result.push(Box::new(armature));
    }

    fn read_joints(
        joints: &mut HashMap<usize, (Id<Joint>, Joint)>,
        node: &gltf::Node,
        parent_id: Option<Id<Joint>>,
    ) {
        let id = Id::new();

        let (translation, rotation, scale) = node.transform().decomposed();
        let local_bind_transform = Transform3D::new(
            Vec3::from(translation),
            // Quat::new(rotation[3], rotation[0], rotation[1], rotation[2]),
            Quat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]),
            Vec3::from(scale),
        );
        let index = node.index();
        let joint = Joint {
            parent_id,
            local_bind_transform,
            ..Default::default()
        };

        joints.insert(index, (id, joint));

        for child in node.children() {
            Self::read_joints(joints, &child, Some(id));
        }
    }

    fn read_mesh(
        output: &mut Output,
        primitive: &gltf::Primitive,
        buffers: &[Vec<u8>],
        name: &str,
    ) {
        let primitive_index = primitive.index();
        if output.loaded_meshes.contains_key(&primitive_index) {
            return;
        }

        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        let mode = primitive.mode();

        if mode != gltf::mesh::Mode::Triangles {
            log::error!("Unsupported topology: {mode:?}");
            return;
        };

        let mut mesh = Mesh::new([name, "mesh"].join("::"));

        let indices = reader
            .read_indices()
            .map(|i| i.into_u32().collect::<Vec<u32>>());

        if let Some(indices) = indices.as_ref().cloned() {
            mesh.set_indices(indices);
        }

        if let Some(positions) = reader
            .read_positions()
            .map(|p| p.collect::<Vec<[f32; 3]>>())
        {
            mesh.set_vertices::<VertexPosition>(positions);
        }

        if let Some(normals) = reader.read_normals().map(|n| n.collect::<Vec<[f32; 3]>>()) {
            mesh.set_vertices::<VertexNormal>(normals);
        }

        if let Some(uvs) = reader.read_tex_coords(0) {
            mesh.set_vertices::<VertexTexture>(uvs.into_f32().collect::<Vec<_>>());
        }

        if let Some(weights) = reader.read_weights(0) {
            mesh.set_vertices::<VertexWeights>(weights.into_f32().collect::<Vec<[f32; 4]>>());
        }

        if let Some(joints) = reader.read_joints(0) {
            mesh.set_vertices::<VertexJoints>(joints.into_u16().collect::<Vec<[u16; 4]>>());
        }

        output
            .loaded_meshes
            .insert(primitive_index, output.result.len());
        output.result.push(Box::new(mesh));
    }

    fn read_material(
        output: &mut Output,
        material: &gltf::Material,
        buffers: &[Vec<u8>],
        name: &str,
    ) {
        let material_index = material.index();
        let already_loaded = material_index
            .map(|index| output.loaded_materials.contains_key(&index))
            .unwrap_or(false);

        if already_loaded {
            return;
        }
        let pbr = material.pbr_metallic_roughness();

        let albedo = Color::from(pbr.base_color_factor());
        let metallic_factor = pbr.metallic_factor();
        let roughness_factor = pbr.roughness_factor();

        let albedo_map = pbr
            .base_color_texture()
            .map(|info| Self::read_image(output, &info.texture(), buffers, name))
            .unwrap_or_default();

        let normal_map = material
            .normal_texture()
            .map(|normals| Self::read_image(output, &normals.texture(), buffers, name))
            .unwrap_or_default();

        let occlusion_map = material
            .normal_texture()
            .map(|occlusion| Self::read_image(output, &occlusion.texture(), buffers, name))
            .unwrap_or_default();

        let asset_name = [name, material.name().unwrap_or("material")].join("::");
        let material_asset = Material {
            name: asset_name,
            albedo,
            albedo_map,
            normal_map,
            occlusion_map,
            metallic_factor,
            roughness_factor,

            ..Default::default()
        };

        if let Some(index) = material_index {
            output.loaded_materials.insert(index, output.result.len());
        }
        output.result.push(Box::new(material_asset));
    }

    fn read_image(
        output: &mut Output,
        texture: &gltf::Texture,
        buffers: &[Vec<u8>],
        name: &str,
    ) -> Id<Image> {
        let asset_name = [name, texture.name().unwrap_or("material")].join("::");
        let image_index = texture.index();
        let mut result_index = output.loaded_images.get(&image_index).cloned();

        if result_index.is_none() {
            let source = texture.source().source();
            let (data, format) = match source {
                gltf::image::Source::Uri { uri, .. } => {
                    const URI_IMAGE_PNG: &str = "data:image/png;base64,";

                    if !uri.starts_with(URI_IMAGE_PNG) {
                        log::warn!("Unsupported texture uri");
                        return Id::default();
                    }

                    match base64_decode(&uri[URI_IMAGE_PNG.len()..]) {
                        Ok(data) => (data, ImageFormat::Png),
                        Err(err) => {
                            log::error!("Could not decode texture data: {err:?}");
                            return Id::default();
                        }
                    }
                }

                gltf::image::Source::View { view, mime_type } => {
                    if mime_type != "image/png" {
                        log::warn!("Unsupported mime: {mime_type}");
                        return Id::default();
                    }

                    let index = view.buffer().index();
                    let offset = view.offset();
                    let tail = offset + view.length();
                    let data = &buffers[index][offset..tail];

                    (data.to_vec(), ImageFormat::Png)
                }
            };

            if let Some(image) = ImageLoader::read_buffer(asset_name, &data, format) {
                let index = output.result.len();
                result_index.replace(index);
                output.loaded_images.insert(image_index, index);
                output.result.push(Box::new(image));
            }
        }

        Id::new()
    }

    fn read_animation(
        output: &mut Output,
        animation: &gltf::Animation,
        buffers: &[Vec<u8>],
        name: &str,
    ) {
        let asset_name = animation
            .name()
            .map(|animation_name| [name, animation_name].join("::"))
            .unwrap_or_else(|| format!("{name}::animation[{}]", animation.index()));

        log::info!("importing animation as `{asset_name}`");

        let mut asset = Animation::new(asset_name);

        for channel in animation.channels() {
            let sampler = channel.sampler();
            let interpolation = match sampler.interpolation() {
                gltf::animation::Interpolation::CubicSpline => Interpolation::CubicSpline,
                gltf::animation::Interpolation::Step => Interpolation::Step,
                gltf::animation::Interpolation::Linear => Interpolation::Linear,
            };
            let index = channel.target().node().index();
            let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
            let outputs = reader.read_outputs();
            let timestamps = reader.read_inputs().unwrap().collect::<Vec<f32>>();
            if let Some(joint_id) = output.loaded_joints.get(&index).cloned() {
                match outputs.unwrap() {
                    gltf::animation::util::ReadOutputs::Translations(out) => asset
                        .add_translation_channel(
                            joint_id,
                            interpolation,
                            timestamps,
                            out.map(Vec3::from).collect(),
                        ),
                    gltf::animation::util::ReadOutputs::Rotations(out) => asset
                        .add_rotation_channel(
                            joint_id,
                            interpolation,
                            timestamps,
                            out.into_f32()
                                .map(|q| Quat::from_xyzw(q[0], q[1], q[2], q[3]))
                                .collect(),
                        ),
                    gltf::animation::util::ReadOutputs::Scales(out) => asset.add_scale_channel(
                        joint_id,
                        interpolation,
                        timestamps,
                        out.map(Vec3::from).collect(),
                    ),
                    gltf::animation::util::ReadOutputs::MorphTargetWeights(ref _weights) => (),
                };
            } else {
                log::warn!(
                    "Animation {} refers target joint ({index}), that does not exist",
                    asset.name()
                );
            }
        }

        output.result.push(Box::new(asset));
    }
}

fn base64_decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::Engine;
    let engine = base64::engine::general_purpose::STANDARD;
    engine.decode(input)
}
