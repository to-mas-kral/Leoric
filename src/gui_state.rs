pub struct GuiState {
    pub selected_node: Option<u32>,
    /// Default 0 (assuming that there is at least 1 model in the scene)
    pub selected_model: usize,
    /// If joints should e visible inside of the mesh
    pub debug_joints: bool,
}

impl GuiState {
    pub fn new() -> Self {
        Self {
            selected_node: None,
            selected_model: 0,
            debug_joints: true,
        }
    }
}
