use ash::vk;
use std::marker::PhantomData;


pub struct GameObject {

}

/// M: Material data that needs to be updated per material
/// O: Object data that needs to be updated for every object
pub struct Pipeline<M, O> {
    pipeline: vk::Pipeline,
    _marker: PhantomData<(M, O)>,
}




