pub mod account_allocator;
use account_allocator::*;

pub struct FuzzContext {}

impl FuzzContext {
    pub fn new() -> Self {
        let _allocator = AccountAllocator::new();

        Self {}
    }
}
