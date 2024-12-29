use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct ResourceUsage: u32 {
        const READ = 1;
        const WRITE = 2;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct ResourceAccess: u32 {
        const HOST_VISIBLE = 1;
        const DEVICE_VISIBLE = 2;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ResourceType {
    Buffer,
    Image,
}

#[derive(Debug, Clone, Copy)]
pub struct ResourceDescriptor {
    pub ty: ResourceType,
    pub usage: ResourceUsage,
    pub access: ResourceAccess,
}
