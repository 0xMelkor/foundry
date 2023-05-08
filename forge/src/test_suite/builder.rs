use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::TestOptions;
use ethers::{
    abi::{Abi, Address},
    types::{Bytes, U256},
};
use foundry_common::{ContractsByArtifact, TestFilter};
use foundry_evm::executor::Executor;

use super::TestSuite;


#[derive(Default)]
pub(crate) struct TestSuiteBuilder {
    id: Option<String>,
    executor: Option<Executor>,
    contract: Option<Abi>,
    deploy_code: Option<Bytes>,
    libs: Option<Vec<Bytes>>,
    test_options: Option<TestOptions>,
    timeout: Option<Duration>,
    initial_balance: Option<U256>,
    sender: Option<Address>,
    errors: Option<Abi>,
    known_contracts: Option<ContractsByArtifact>,
}

impl TestSuiteBuilder {
    #[must_use]
    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    #[must_use]
    pub fn executor(mut self, e: Executor) -> Self {
        self.executor = Some(e);
        self
    }

    #[must_use]
    pub fn contract(mut self, c: Abi) -> Self {
        self.contract = Some(c);
        self
    }

    #[must_use]
    pub fn deploy_code(mut self, dc: Bytes) -> Self {
        self.deploy_code = Some(dc);
        self
    }

    #[must_use]
    pub fn libs(mut self, libs: Vec<Bytes>) -> Self {
        self.libs = Some(libs);
        self
    }

    #[must_use]
    pub fn test_options(mut self, opts: TestOptions) -> Self {
        self.test_options = Some(opts);
        self
    }

    #[must_use]
    pub fn timeout(mut self, t: Duration) -> Self {
        self.timeout = Some(t);
        self
    }

    #[must_use]
    pub fn initial_balance(mut self, b: U256) -> Self {
        self.initial_balance = Some(b);
        self
    }

    #[must_use]
    pub fn sender(mut self, s: Option<Address>) -> Self {
        self.sender = s;
        self
    }

    #[must_use]
    pub fn errors(mut self, e: Option<Abi>) -> Self {
        self.errors = e;
        self
    }

    #[must_use]
    pub fn known_contracts(mut self, kc: ContractsByArtifact) -> Self {
        self.known_contracts = Some(kc);
        self
    }

    #[must_use]
    pub fn build(self, filter: impl TestFilter + 'static) -> TestSuite<impl TestFilter> {
        TestSuite {
            id: self.id.expect("Id"),
            executor: self.executor.expect("Executor"),
            contract: self.contract.expect("Contract"),
            deploy_code: self.deploy_code.expect("Deploy code"),
            libs: self.libs.unwrap_or_default(),
            filter,
            test_options: self.test_options.expect("Test options"),
            timeout: self.timeout.expect("Timeout"),
            complete: Arc::new(Mutex::new(false)),
            initial_balance: self.initial_balance.expect("Initial balance"),
            sender: self.sender,
            errors: self.errors,
            known_contracts: self.known_contracts.expect("Known Contracts"),
        }
    }
}
