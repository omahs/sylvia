use cosmwasm_std::{Response, StdError, StdResult};
use cw_storage_plus::Item;
use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx};
use sylvia::{contract, schemars};

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

use crate::messages::CountResponse;

pub struct CounterContract {
    pub(crate) counter: Item<'static, u32>,
}

#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
#[sv::override_entry_point(sudo=crate::entry_points::sudo(crate::messages::SudoMsg))]
#[sv::override_entry_point(exec=crate::entry_points::execute(crate::messages::CustomExecMsg))]
impl CounterContract {
    pub const fn new() -> Self {
        Self {
            counter: Item::new("counter"),
        }
    }

    #[msg(instantiate)]
    pub fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
        self.counter.save(ctx.deps.storage, &0)?;
        Ok(Response::new())
    }

    #[msg(query)]
    pub fn count(&self, ctx: QueryCtx) -> StdResult<CountResponse> {
        let count = self.counter.load(ctx.deps.storage)?;
        Ok(CountResponse { count })
    }

    #[msg(exec)]
    pub fn increase_by_two(&self, ctx: ExecCtx) -> StdResult<Response> {
        self.counter
            .update(ctx.deps.storage, |count| -> Result<u32, StdError> {
                Ok(count + 2)
            })?;
        Ok(Response::new())
    }
}
