
#[cfg(test)]
mod tests {
    use crate::msg::{ListPendingResp, ListSignedResp};

    use super::*;
    use cosmwasm_std::{coins, Addr, Coin};
    use cw_multi_test::{App, ContractWrapper, Executor};

    fn instantiate_contract() -> (Addr, App) {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("owner"), coins(5, "atom"))
                .unwrap();
        });

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let coin = Coin::new(5, "atom");

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    owners: vec![
                        Addr::unchecked("owner1"),
                        Addr::unchecked("owner2"),
                        Addr::unchecked("owner3"),
                    ],
                    quorum: 2,
                },
                &[coin],
                "Multisig",
                None,
            )
            .unwrap();

        (addr, app)
    }

    #[test]
    fn test_instantiate() {
        let (addr, app) = instantiate_contract();

        let balance: Vec<Coin> = app.wrap().query_all_balances(&addr).unwrap();
        assert_eq!(vec![Coin::new(5, "atom")], balance);
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_propose_unauthorized() {
        let (addr, mut app) = instantiate_contract();

        let msg = ExecuteMsg::CreateTransaction {
            to: Addr::unchecked("owner"),
            coins: vec![Coin::new(5, "atom")],
        };
        app.execute_contract(Addr::unchecked("unathorized"), addr.clone(), &msg, &[])
            .unwrap();
    }

    #[test]
    fn test_propose() {
        let (addr, mut app) = instantiate_contract();

        let msg = ExecuteMsg::CreateTransaction {
            to: Addr::unchecked("owner"),
            coins: vec![Coin::new(5, "atom")],
        };
        app.execute_contract(Addr::unchecked("owner1"), addr.clone(), &msg, &[])
            .unwrap();
        let msg = QueryMsg::ListPending {};

        let resp: ListPendingResp = app.wrap().query_wasm_smart(addr, &msg).unwrap();
        let mut tx = Transaction::new(Addr::unchecked("owner"), 0, vec![Coin::new(5, "atom")]);
        tx.num_confirmations = 1;
        assert_eq!(&tx, resp.transactions.index(0).unwrap());
    }

    #[test]
    #[should_panic(expected = "You already signed transaction with id: 0")]
    fn test_sign_after_already_signed() {
        let (addr, mut app) = instantiate_contract();

        let msg = ExecuteMsg::CreateTransaction {
            to: Addr::unchecked("owner"),
            coins: vec![Coin::new(5, "atom")],
        };
        app.execute_contract(Addr::unchecked("owner1"), addr.clone(), &msg, &[])
            .unwrap();

        let msg = ExecuteMsg::SignTransactions { tx_id: 0 };

        app.execute_contract(Addr::unchecked("owner1"), addr.clone(), &msg, &[])
            .unwrap();
    }

    #[test]
    fn test_sign() {
        let (addr, mut app) = instantiate_contract();

        let msg = ExecuteMsg::CreateTransaction {
            to: Addr::unchecked("owner"),
            coins: vec![Coin::new(5, "atom")],
        };
        app.execute_contract(Addr::unchecked("owner1"), addr.clone(), &msg, &[])
            .unwrap();

        let msg = ExecuteMsg::SignTransactions { tx_id: 0 };

        app.execute_contract(Addr::unchecked("owner2"), addr.clone(), &msg, &[])
            .unwrap();
        app.execute_contract(Addr::unchecked("owner3"), addr.clone(), &msg, &[])
            .unwrap();

        let resp_owner1: ListSignedResp = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::ListSigned {
                    admin: Addr::unchecked("owner2"),
                    tx_id: 0,
                },
            )
            .unwrap();

        let resp_owner2: ListSignedResp = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::ListSigned {
                    admin: Addr::unchecked("owner2"),
                    tx_id: 0,
                },
            )
            .unwrap();

        let resp_owner3: ListSignedResp = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::ListSigned {
                    admin: Addr::unchecked("owner3"),
                    tx_id: 0,
                },
            )
            .unwrap();

        assert_eq!(resp_owner1.signed, true);
        assert_eq!(resp_owner2.signed, true);
        assert_eq!(resp_owner3.signed, true);

        let resp: ListPendingResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::ListPending {})
            .unwrap();

        assert_eq!(resp.transactions.index(0).unwrap().num_confirmations, 3);
    }

    #[test]
    #[should_panic(
        expected = "Not enough admins signed this transaction, the quorum is 2 and only 1 signed the transaction"
    )]
    fn test_execute_under_quorum() {
        let (addr, mut app) = instantiate_contract();

        let msg = ExecuteMsg::CreateTransaction {
            to: Addr::unchecked("owner"),
            coins: vec![Coin::new(5, "atom")],
        };
        app.execute_contract(Addr::unchecked("owner1"), addr.clone(), &msg, &[])
            .unwrap();

        let msg = ExecuteMsg::ExecuteTransaction { tx_id: 0 };

        app.execute_contract(Addr::unchecked("owner1"), addr.clone(), &msg, &[])
            .unwrap();
    }

    #[test]
    fn test_execute() {
        let (addr, mut app) = instantiate_contract();

        let msg = ExecuteMsg::CreateTransaction {
            to: Addr::unchecked("owner"),
            coins: vec![Coin::new(5, "atom")],
        };
        app.execute_contract(Addr::unchecked("owner1"), addr.clone(), &msg, &[])
            .unwrap();

        let msg = ExecuteMsg::SignTransactions { tx_id: 0 };

        app.execute_contract(Addr::unchecked("owner2"), addr.clone(), &msg, &[])
            .unwrap();
        app.execute_contract(Addr::unchecked("owner3"), addr.clone(), &msg, &[])
            .unwrap();

        let resp_owner1: ListSignedResp = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::ListSigned {
                    admin: Addr::unchecked("owner2"),
                    tx_id: 0,
                },
            )
            .unwrap();

        let resp_owner2: ListSignedResp = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::ListSigned {
                    admin: Addr::unchecked("owner2"),
                    tx_id: 0,
                },
            )
            .unwrap();

        let resp_owner3: ListSignedResp = app
            .wrap()
            .query_wasm_smart(
                addr.clone(),
                &QueryMsg::ListSigned {
                    admin: Addr::unchecked("owner3"),
                    tx_id: 0,
                },
            )
            .unwrap();

        assert_eq!(resp_owner1.signed, true);
        assert_eq!(resp_owner2.signed, true);
        assert_eq!(resp_owner3.signed, true);

        let msg = ExecuteMsg::ExecuteTransaction { tx_id: 0 };

        app.execute_contract(Addr::unchecked("owner3"), addr.clone(), &msg, &[])
            .unwrap();

        let balance: Coin = app.wrap().query_balance(&addr, "atom").unwrap();
        assert_eq!(Coin::new(0, "atom"), balance);

        let balance: Coin = app
            .wrap()
            .query_balance(&Addr::unchecked("owner"), "atom")
            .unwrap();
        assert_eq!(Coin::new(5, "atom"), balance);
    }
}