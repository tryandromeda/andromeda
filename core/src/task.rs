/// An Id representing a Task.
#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub struct TaskId(u32);

impl TaskId {
    pub fn index(&self) -> u32 {
        self.0
    }

    pub fn from_index(index: u32) -> Self {
        Self(index)
    }
}
