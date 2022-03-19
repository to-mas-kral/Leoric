pub struct GuiState {
    pub selected_node: Option<u32>,
}

impl GuiState {
    pub fn new() -> Self {
        Self {
            selected_node: None,
        }
    }
}
