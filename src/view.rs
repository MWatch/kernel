
use super::Ssd1351;

/// Example Home is a model for the view, that includes all the data
/// implementing the Renderable trait provides the impl to actually draw
/// the scene on the hardware. This trait can be implemented on sub components
/// meaning we can break down complex views.
/// 
/// pub struct Home {
///     menu: HomeSubMenu
/// }
/// pub struct HomeSubMenu {}
/// 
/// impl Renderable for Home {
///     fn render(&mut self, disp: &mut Ssd1351) {
///         // we can break down the rendering into there own components
///         self.menu.render(disp);
///         // render other stuff
///     }
/// }
/// 
/// impl Renderable for HomeSubMenu {
///     fn render(&mut self, disp: &mut Ssd1351){
///         
///     }
/// }
pub trait Renderable {
    fn render(&mut self, &mut Ssd1351);
}
