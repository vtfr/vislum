use vislum_render::{context::RenderContextBuilder, resources::texture::{TextureDescription, TextureDimensions}};

fn main() {
    let mut context = RenderContextBuilder::auto();
    let texture = context.resource_manager_mut().create_texture(TextureDescription {
        dimensions: TextureDimensions {
            width: 1920,
            height: 1080,
        },
    });

    println!("Context: {:#?}", context);
}
