use std::time::Instant;

use eyre::{eyre, Result};
use glam::{Quat, Vec3};
use gltf::animation::{
    util::{ReadOutputs, Rotations},
    Interpolation,
};

use super::DataBundle;

pub struct Animations {
    pub animations: Vec<Animation>,
    pub animation_control: AnimationControl,
}

pub enum AnimationControl {
    Loop {
        active_animation: usize,
        start_time: Instant,
    },
    Controllable {
        active_animation: usize,
    },
    Static,
}

impl AnimationControl {}

/// Contains all animation data
pub struct Animation {
    pub channels: Vec<Channel>,
    /// Current time of the animation
    pub current_time: f32,
    /// The time in seconds of the last keyframe, start time is implicitly 0
    pub end_time: f32,
    pub name: Option<String>,
}

impl Animation {
    pub fn new(
        channels: Vec<Channel>,
        current_time: f32,
        end_time: f32,
        name: Option<String>,
    ) -> Self {
        Self {
            channels,
            current_time,
            end_time,
            name,
        }
    }

    pub fn from_gltf(gltf: &gltf::Document, bundle: &DataBundle) -> Result<Animations> {
        let mut animations = Vec::new();

        for animation in gltf.animations() {
            let mut channels = Vec::new();

            for channel in animation.channels() {
                let node_index = channel.target().node().index();

                let reader = channel.reader(|buf| Some(&bundle.buffers[buf.index()]));
                let keyframe_times: Vec<f32> = reader
                    .read_inputs()
                    .ok_or(eyre!("Animation channel doesn't contain keyframe times"))?
                    .collect();

                let transforms = match reader
                    .read_outputs()
                    .ok_or(eyre!("Animation channel doesn't contain transforms"))?
                {
                    ReadOutputs::Translations(trans) => {
                        let data: Vec<Vec3> = trans.map(|v| Vec3::from(v)).collect();
                        AnimationTransforms::Translations(data)
                    }
                    ReadOutputs::Scales(scales) => {
                        let data: Vec<Vec3> = scales.map(|v| Vec3::from(v)).collect();
                        AnimationTransforms::Scales(data)
                    }
                    ReadOutputs::Rotations(rotations) => Self::decode_rotations(rotations),
                    ReadOutputs::MorphTargetWeights(_) => todo!(),
                };

                let interpolation_type = channel.sampler().interpolation();

                let channel =
                    Channel::new(node_index, keyframe_times, transforms, interpolation_type);
                channels.push(channel);
            }

            let name = animation.name().map(|n| n.to_string());

            // Sorting floating point numbers in Rust is cumbersome, partly because of NaN
            let end_time = channels
                .iter()
                .map(|c| *c.keyframe_times.last().unwrap_or(&0.))
                .fold(0f32, |a, b| a.max(b));
            let animation = Animation::new(channels, 0.1, end_time, name);

            animations.push(animation);
        }

        Ok(Animations {
            animations,
            animation_control: AnimationControl::Static,
        })
    }

    /// https://www.khronos.org/registry/glTF/specs/2.0/glTF-2.0.html#animations
    /// Implementations MUST use following equations to decode real floating-point
    /// value f from a normalized integer c and vise-versa:
    ///
    /// accessor.componentType 	    int-to-float 	                float-to-int
    /// ------------------------------------------------------------------------------
    /// signed byte                 f = max(c / 127.0, -1.0)        c = round(f * 127.0)
    /// unsigned byte               f = c / 255.0                   c = round(f * 255.0)
    /// signed short                f = max(c / 32767.0, -1.0)      c = round(f * 32767.0)
    /// unsigned short              f = c / 65535.0                 c = round(f * 65535.0)
    fn decode_rotations(rotations: Rotations) -> AnimationTransforms {
        let data: Vec<[f32; 4]> = match rotations {
            Rotations::I8(r) => r.map(|v| v.map(|s| (s as f32 / 127.).max(-1.))).collect(),
            Rotations::U8(r) => r.map(|v| v.map(|s| (s as f32 / 255.))).collect(),
            Rotations::I16(r) => r.map(|v| v.map(|s| (s as f32 / 32767.).max(-1.))).collect(),
            Rotations::U16(r) => r.map(|v| v.map(|s| (s as f32 / 65535.))).collect(),
            Rotations::F32(r) => r.collect(),
        };

        let data = data.iter().map(|arr| Quat::from_array(*arr)).collect();

        AnimationTransforms::Rotations(data)
    }
}

pub struct Channel {
    /// Index of the node this channel is applied to
    pub node: usize,
    /// Times of the keyframes
    pub keyframe_times: Vec<f32>,
    /// Transforms that should be applied to the respective node
    pub transforms: AnimationTransforms,
    /// The type of the interpolation that should be applied between the keyframes
    pub interpolation_type: Interpolation,
}

impl Channel {
    pub fn new(
        node_index: usize,
        keyframe_times: Vec<f32>,
        transforms: AnimationTransforms,
        interpolation_type: Interpolation,
    ) -> Self {
        Self {
            node: node_index,
            keyframe_times,
            transforms,
            interpolation_type,
        }
    }

    pub fn get_fixed_transform(&self, index: usize) -> AnimationTransform {
        match self.interpolation_type {
            Interpolation::Linear => {}
            Interpolation::Step => todo!("Step interpolation"),
            Interpolation::CubicSpline => todo!("Cubic spline interpolation"),
        }

        match &self.transforms {
            AnimationTransforms::Translations(trans) => {
                AnimationTransform::Translation(trans[index])
            }
            AnimationTransforms::Rotations(rotations) => {
                AnimationTransform::Rotation(rotations[index])
            }
            AnimationTransforms::Scales(scales) => AnimationTransform::Scale(scales[index]),
        }
    }

    /// https://www.khronos.org/registry/glTF/specs/2.0/glTF-2.0.html#appendix-c-interpolation
    pub fn interpolate_transforms(
        &self,
        start_index: usize, // end index is always start_index + 1
        coeff: f32,
    ) -> AnimationTransform {
        match self.interpolation_type {
            Interpolation::Linear => {}
            Interpolation::Step => todo!("Step interpolation"),
            Interpolation::CubicSpline => todo!("Cubic spline interpolation"),
        }

        match &self.transforms {
            AnimationTransforms::Translations(trans) => {
                let start = trans[start_index];
                let end = trans[start_index + 1];

                let interpolated = start.lerp(end, coeff);
                return AnimationTransform::Translation(interpolated);
            }
            AnimationTransforms::Rotations(rotations) => {
                let start = rotations[start_index].normalize();
                let end = rotations[start_index + 1].normalize();

                let interpolated = if start.dot(end) > 0. {
                    start.slerp(end, coeff)
                } else {
                    (-start).slerp(end, coeff)
                };

                return AnimationTransform::Rotation(interpolated.normalize());
            }
            AnimationTransforms::Scales(scales) => {
                let start = scales[start_index];
                let end = scales[start_index + 1];

                let interpolated = start.lerp(end, coeff);
                return AnimationTransform::Scale(interpolated);
            }
        }
    }
}

pub enum AnimationTransforms {
    Translations(Vec<Vec3>),
    Rotations(Vec<Quat>),
    Scales(Vec<Vec3>),
}

pub enum AnimationTransform {
    Translation(Vec3),
    Rotation(Quat),
    Scale(Vec3),
}
