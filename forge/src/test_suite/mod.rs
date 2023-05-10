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

#[derive(Debug)]
pub enum RunEvent {
    Slow(Payload<Duration>),
    Success(Payload<SuiteResult>),
    Error(Payload<eyre::ErrReport>),
}

#[derive(Debug)]
pub struct Payload<T: 'static> {
    pub id: String,
    pub body: T,
}

pub(crate) mod builder;
pub(crate) struct TestSuite<F: TestFilter + 'static> {
    pub id: String,
    pub executor: Executor,
    pub contract: Abi,
    pub deploy_code: Bytes,
    pub libs: Vec<Bytes>,
    pub filter: F,
    pub test_options: TestOptions,
    pub timeout: Duration,
    pub complete: Arc<Mutex<bool>>,
    pub initial_balance: U256,
    pub sender: Option<Address>,
    pub errors: Option<Abi>,
    pub known_contracts: ContractsByArtifact,
}

impl<F: TestFilter + 'static> TestSuite<F> {
    /// TODO docs
    pub fn run(self, tx: SyncSender<RunEvent>) {
        let id = self.id.clone();
        let complete = self.complete.clone();
        let timeout = self.timeout;
        let timeout_secs = timeout.as_secs();
        let tx_1: SyncSender<RunEvent> = tx.clone();
        // TODO DOCS
        let mut slow_count = 0;
        rayon::spawn(move || loop {
            if !*complete.lock().unwrap() {
                std::thread::sleep(timeout);
                let elapsed = timeout_secs * (1 + slow_count);
                let body = Duration::from_secs(elapsed);
                let evt = RunEvent::Slow(Payload { id: id.clone(), body });
                tx_1.send(evt).unwrap();
            } else {
                break;
            }
        });

        // TODO DOCS
        let id = self.id.clone();
        rayon::spawn(move || {
            match self.runner().run_tests(
                &self.filter,
                self.test_options,
                Some(&self.known_contracts),
            ) {
                Ok(body) => tx.send(RunEvent::Success(Payload { id, body })).unwrap(),
                Err(body) => tx.send(RunEvent::Error(Payload { id, body })).unwrap(),
            }

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
