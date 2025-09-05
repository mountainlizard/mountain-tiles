#[derive(Clone, Copy, PartialEq, Default)]
pub enum Mode {
    Select,
    #[default]
    Draw,
    Erase,
}
