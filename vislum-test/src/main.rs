use vislum_render::context::RenderContextBuilder;

fn main() {
    let context = RenderContextBuilder::auto();
    println!("Context: {:#?}", context);
}
