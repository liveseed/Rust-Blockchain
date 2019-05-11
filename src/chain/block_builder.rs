use super::{Error, SharedBackend};
use crate::traits::{
	Backend, BuilderExecutor, AsExternalities,
	Block, ChainQuery, Auxiliary
};

/// Block builder.
pub struct BlockBuilder<'a, E: BuilderExecutor, Ba: Backend> {
	executor: &'a E,
	pending_block: E::BuildBlock,
	pending_state: Ba::State,
}

impl<'a, E: BuilderExecutor, Ba> BlockBuilder<'a, E, Ba> where
	Ba: Backend<Block=E::Block> + ChainQuery,
	<Ba as Backend>::Auxiliary: Auxiliary<E::Block>,
	<Ba as Backend>::State: AsExternalities<E::Externalities>,
{
	/// Create a new block builder.
	pub fn new(backend: &SharedBackend<Ba>, executor: &'a E, parent_hash: &<E::Block as Block>::Identifier, inherent: E::Inherent) -> Result<Self, Error> {
		let parent_block = backend.read().block_at(parent_hash)
			.map_err(|e| Error::Backend(Box::new(e)))?;

		let mut pending_state = backend.read().state_at(parent_hash)
			.map_err(|e| Error::Backend(Box::new(e)))?;

		let pending_block = executor.initialize_block(
			&parent_block,
			pending_state.as_externalities(),
			inherent
		).map_err(|e| Error::Executor(Box::new(e)))?;

		Ok(Self {
			executor, pending_block, pending_state,
		})
	}

	/// Apply extrinsic to the block builder.
	pub fn apply_extrinsic(&mut self, extrinsic: E::Extrinsic) -> Result<(), Error> {
		self.executor.apply_extrinsic(
			&mut self.pending_block,
			extrinsic,
			self.pending_state.as_externalities()
		).map_err(|e| Error::Executor(Box::new(e)))
	}

	/// Finalize the block.
	pub fn finalize(mut self) -> Result<(E::BuildBlock, Ba::State), Error> {
		self.executor.finalize_block(
			&mut self.pending_block,
			self.pending_state.as_externalities()
		).map_err(|e| Error::Executor(Box::new(e)))?;

		Ok((self.pending_block, self.pending_state))
	}
}
