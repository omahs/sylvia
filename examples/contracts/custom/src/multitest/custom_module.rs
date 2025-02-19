use cosmwasm_schema::schemars::JsonSchema;
use cosmwasm_std::testing::{MockApi, MockStorage};
use cosmwasm_std::{
    to_binary, Addr, Api, Binary, BlockInfo, CustomQuery, Empty, Querier, StdError, StdResult,
    Storage,
};
use cw_multi_test::{AppResponse, BankKeeper, CosmosRouter, Module, WasmKeeper};
use cw_storage_plus::Item;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::messages::{CountResponse, CounterMsg, CounterQuery};

pub type CustomApp = cw_multi_test::App<
    BankKeeper,
    MockApi,
    MockStorage,
    CustomModule,
    WasmKeeper<CounterMsg, CounterQuery>,
>;

pub struct CustomModule {
    pub counter: Item<'static, u64>,
}

impl CustomModule {
    pub fn new() -> Self {
        Self {
            counter: Item::new("counter"),
        }
    }

    pub fn save_counter(&self, storage: &mut dyn Storage, value: u64) -> StdResult<()> {
        self.counter.save(storage, &value)
    }
}

impl Module for CustomModule {
    type ExecT = CounterMsg;
    type QueryT = CounterQuery;
    type SudoT = Empty;

    fn execute<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        _sender: Addr,
        msg: Self::ExecT,
    ) -> anyhow::Result<AppResponse>
    where
        ExecC: Debug + Clone + PartialEq + JsonSchema + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        match msg {
            CounterMsg::Increment {} => {
                self.counter
                    .update(storage, |value| Ok::<_, StdError>(value + 1))?;
                Ok(AppResponse::default())
            }
        }
    }

    fn sudo<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        _msg: Self::SudoT,
    ) -> anyhow::Result<AppResponse>
    where
        ExecC: Debug + Clone + PartialEq + JsonSchema + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        Ok(AppResponse::default())
    }

    fn query(
        &self,
        _api: &dyn Api,
        storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        request: Self::QueryT,
    ) -> anyhow::Result<Binary> {
        match request {
            CounterQuery::Count {} => {
                let count = self.counter.load(storage)?;
                let res = CountResponse { count };
                to_binary(&res).map_err(Into::into)
            }
        }
    }
}
