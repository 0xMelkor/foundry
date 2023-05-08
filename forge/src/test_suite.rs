use std::{
    sync::{mpsc::SyncSender, Arc, Mutex},
    time::Duration,
};

use crate::{result::SuiteResult, ContractRunner, TestOptions};
use ethers::{
    abi::{Abi, Address},
    types::{Bytes, U256},
};
use foundry_common::{ContractsByArtifact, TestFilter};
use foundry_evm::executor::Executor;

pub enum TestRunEvent {
    ExecutionSlow(String),
    ExecutionFinished(eyre::Result<SuiteResult>),
}

pub(crate) struct TestSuite<F: TestFilter + 'static> {
    pub id: String,
    pub executor: Executor,
    pub contract: Abi,
    pub deploy_code: Bytes,
    pub libs: Vec<Bytes>,
    pub filter: F,
    pub test_opts: TestOptions,
    pub timeout: Duration,
    pub complete: Arc<Mutex<bool>>,
    pub initial_balance: U256,
    pub sender: Option<Address>,
    pub errors: Option<Abi>,
    pub known_contracts: ContractsByArtifact,
}

impl<F: TestFilter + 'static> TestSuite<F> {
    pub fn run(self, tx: SyncSender<TestRunEvent>) {
        let id = self.id.clone();
        let complete = self.complete.clone();
        let timeout = self.timeout;
        let tx_1: SyncSender<TestRunEvent> = tx.clone();

        // TODO DOCS
        rayon::spawn(move || loop {
            if !*complete.lock().unwrap() {
                std::thread::sleep(timeout);
                let evt = TestRunEvent::ExecutionSlow(id.clone());
                tx_1.send(evt).unwrap();
            } else {
                break;
            }
        });

        // TODO DOCS
        rayon::spawn(move || {
            let runner = self.runner();
            let o = self.test_opts;
            let f = &self.filter;
            let kc = Some(&self.known_contracts);
            let result = runner.run_tests(f, o, kc);

            let evt = TestRunEvent::ExecutionFinished(result);
            tx.send(evt).unwrap();
            *self.complete.lock().unwrap() = true;
        });
    }

    fn runner(&self) -> ContractRunner {
        ContractRunner::new(
            self.executor.clone(),
            &self.contract,
            self.deploy_code.clone(),
            self.initial_balance,
            self.sender,
            self.errors.as_ref(),
            &self.libs,
        )
    }
}
