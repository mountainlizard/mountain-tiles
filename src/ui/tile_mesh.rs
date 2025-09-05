use egui::{epaint::Vertex, Color32, Mesh, Rect};

use crate::geom::transform::Transform;

pub trait TileMesh {
    fn add_rect_with_transform(
        &mut self,
        rect: Rect,
        uv: Rect,
        transform: Transform,
        color: Color32,
    );
}

impl TileMesh for Mesh {
    fn add_rect_with_transform(
        &mut self,
        rect: Rect,
        uv: Rect,
        transform: Transform,
        color: Color32,
    ) {
        #![allow(clippy::identity_op)]

        let idx = self.vertices.len() as u32;
        self.add_triangle(idx + 0, idx + 1, idx + 2);
        self.add_triangle(idx + 2, idx + 1, idx + 3);

        let (uv_left_top, uv_right_top, uv_left_bottom, uv_right_bottom) = match transform {
            Transform::None => (
                uv.left_top(),
                uv.right_top(),
                uv.left_bottom(),
                uv.right_bottom(),
            ),
            Transform::Rotate90 => (
                uv.left_bottom(),
                uv.left_top(),
                uv.right_bottom(),
                uv.right_top(),
            ),
            Transform::Rotate180 => (
                uv.right_bottom(),
                uv.left_bottom(),
                uv.right_top(),
                uv.left_top(),
            ),
            Transform::Rotate270 => (
                uv.right_top(),
                uv.right_bottom(),
                uv.left_top(),
                uv.left_bottom(),
            ),
            Transform::MirrorX => (
                uv.right_top(),
                uv.left_top(),
                uv.right_bottom(),
                uv.left_bottom(),
            ),
            Transform::MirrorXRotate90 => (
                uv.right_bottom(),
                uv.right_top(),
                uv.left_bottom(),
                uv.left_top(),
            ),
            Transform::MirrorXRotate180 => (
                uv.left_bottom(),
                uv.right_bottom(),
                uv.left_top(),
                uv.right_top(),
            ),
            Transform::MirrorXRotate270 => (
                uv.left_top(),
                uv.left_bottom(),
                uv.right_top(),
                uv.right_bottom(),
            ),
        };

        self.vertices.push(Vertex {
            pos: rect.left_top(),
            uv: uv_left_top,
            color,
        });
        self.vertices.push(Vertex {
            pos: rect.right_top(),
            uv: uv_right_top,
            color,
        });
        self.vertices.push(Vertex {
            pos: rect.left_bottom(),
            uv: uv_left_bottom,
            color,
        });
        self.vertices.push(Vertex {
            pos: rect.right_bottom(),
            uv: uv_right_bottom,
            color,
        });
    }
}
