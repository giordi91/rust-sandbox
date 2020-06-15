use winit::event::*;
pub trait Application {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
    // input() won't deal with GPU code, so it can be synchronous
    fn input(&mut self, event: &WindowEvent) -> bool;
    fn update(&mut self);
    fn render(&mut self);
}
