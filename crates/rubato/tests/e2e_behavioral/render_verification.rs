//! Phase 5g: Render verification E2E tests.
//!
//! Tests SpriteBatch vertex output and rendering without GPU.

use rubato::render::sprite_batch::SpriteBatch;

#[test]
fn sprite_batch_starts_with_zero_vertices() {
    let batch = SpriteBatch::new();
    assert_eq!(batch.vertex_count(), 0);
}

#[test]
fn sprite_batch_vertex_count_after_begin_end() {
    let mut batch = SpriteBatch::new();
    batch.begin();
    // No draws between begin/end
    batch.end();
    assert_eq!(
        batch.vertex_count(),
        0,
        "empty batch should have 0 vertices"
    );
}

#[test]
fn sprite_batch_vertices_accessible() {
    let batch = SpriteBatch::new();
    let vertices = batch.vertices();
    assert!(vertices.is_empty(), "new batch should have empty vertices");
}
