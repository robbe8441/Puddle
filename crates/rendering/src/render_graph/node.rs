use super::builder::{GraphContext, NodeBuildContext};

pub trait Node {
    fn create(&mut self, ctx: &mut NodeBuildContext);

    fn execute(&mut self, ctx: &mut GraphContext);

    fn destroy(&mut self) {}
}
