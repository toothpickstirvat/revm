use context::{ContextTr, TxEnv};
use criterion::Criterion;
use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET, BENCH_TARGET_BALANCE};
use revm::{
    bytecode::Bytecode,
    context_interface::JournalTr,
    primitives::{TxKind, U256},
    Context, ExecuteEvm, MainBuilder, MainContext,
};

pub fn run(criterion: &mut Criterion) {
    let mut evm = Context::mainnet()
        .with_db(BenchmarkDB::new_bytecode(Bytecode::new()))
        .modify_cfg_chained(|cfg| cfg.disable_nonce_check = true)
        .build_mainnet();

    let tx = TxEnv {
        caller: BENCH_CALLER,
        kind: TxKind::Call(BENCH_TARGET),
        value: U256::from(1),
        gas_price: 1,
        gas_priority_fee: None,
        ..Default::default()
    };

    evm.ctx.tx = tx.clone();

    let mut i = 0;
    criterion.bench_function("transfer", |b| {
        b.iter_batched(
            || {
                // create a transfer input
                tx.clone()
            },
            |input| {
                i += 1;
                evm.transact_one(input).unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });

    let balance = evm
        .journal_mut()
        .load_account(BENCH_TARGET)
        .unwrap()
        .data
        .info
        .balance;

    if balance != BENCH_TARGET_BALANCE + U256::from(i) {
        panic!("balance of transfers is not correct");
    }

    // drop the journal
    let _ = evm.finalize();

    evm.modify_cfg(|cfg| cfg.disable_nonce_check = false);

    criterion.bench_function("transfer_finalize", |b| {
        b.iter(|| {
            let _ = evm.replay().unwrap();
        })
    });
}
