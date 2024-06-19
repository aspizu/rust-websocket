#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClientID(u64);

#[derive(Default)]
pub struct ClientIDFactory {
    state: u64,
}

impl ClientIDFactory {
    pub fn new() -> Self {
        Self { state: 0 }
    }

    pub fn create_id(&mut self) -> ClientID {
        let id = ClientID(self.state);
        self.state = self.state.overflowing_add(1).0;
        id
    }
}
