use glam::{Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::{
    model::Joint,
    ogl::{self, shader::Shader},
};

// TODO: do not create a new buffer every frame
pub fn draw_joints(world_transforms: &[Mat4], shader: &Shader) {
    let mut positions = Vec::new();
    let texcoords = vec![Vec2::ZERO; world_transforms.len()];
    let normals = vec![Vec3::ZERO; world_transforms.len()];

    for trans in world_transforms {
        let pos = *trans * Vec4::new(0., 0., 0., 1.);
        positions.push(pos.xyz());
    }

    let mut vao = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        let _positions = ogl::create_float_buf(&positions, 3, ogl::POS_INDEX, gl::FLOAT);
        let _texcoords = ogl::create_float_buf(&texcoords, 2, ogl::TEXCOORDS_INDEX, gl::FLOAT);
        let _normals = ogl::create_float_buf(&normals, 3, ogl::NORMALS_INDEX, gl::FLOAT);

        gl::BindVertexArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    shader.render(|| unsafe {
        gl::BindVertexArray(vao);
        gl::PointSize(4.);
        gl::DrawArrays(gl::POINTS, 0, positions.len() as i32);
        gl::BindVertexArray(0);
    });
}

pub fn draw_bones(world_transforms: &[Mat4], joints: &[Joint], shader: &Shader) {
    let mut positions = Vec::new();

    for (i, joint) in joints.iter().enumerate() {
        if let Some(parent) = joint.parent {
            let pos = world_transforms[i] * Vec4::new(0., 0., 0., 1.);
            positions.push(pos.xyz());

            let pos = world_transforms[parent] * Vec4::new(0., 0., 0., 1.);
            positions.push(pos.xyz());
        }
    }

    let texcoords = vec![Vec2::ZERO; positions.len()];
    let normals = vec![Vec3::ZERO; positions.len()];

    let mut vao = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        let _positions = ogl::create_float_buf(&positions, 3, ogl::POS_INDEX, gl::FLOAT);
        let _texcoords = ogl::create_float_buf(&texcoords, 2, ogl::TEXCOORDS_INDEX, gl::FLOAT);
        let _normals = ogl::create_float_buf(&normals, 3, ogl::NORMALS_INDEX, gl::FLOAT);

        gl::BindVertexArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    shader.render(|| unsafe {
        gl::BindVertexArray(vao);
        gl::DrawArrays(gl::LINES, 0, positions.len() as i32);
        gl::BindVertexArray(0);
    });
}
