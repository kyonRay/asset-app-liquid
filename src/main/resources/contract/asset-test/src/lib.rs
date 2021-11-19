#![cfg_attr(not(feature = "std"), no_std)]

use liquid::storage;
use liquid_lang as liquid;
use liquid_lang::InOut;
use liquid_prelude::{
    string::{String, ToString},
    vec::Vec,
};

#[derive(InOut)]
pub struct KVField {
    key: String,
    value: String,
}
#[derive(InOut)]
pub struct Entry {
    fileds: Vec<KVField>,
}

#[derive(InOut)]
pub enum Comparator {
    EQ(u8),
    NE(u8),
    GT(u8),
    GE(u8),
    LT(u8),
    LE(u8),
}

#[derive(InOut)]
pub struct CompareTriple {
    lvalue: String,
    rvalue: String,
    cmp: Comparator,
}

#[derive(InOut)]
pub struct Condition {
    cond_fields: Vec<CompareTriple>,
}

#[liquid::interface(name = auto)]
mod table {
    use super::*;

    extern "liquid" {
        fn createTable(
            &mut self,
            table_name: String,
            key: String,
            value_fields: String,
        ) -> i256;
        fn select(&self, table_name: String, condition: Condition) -> Vec<Entry>;
        fn insert(&mut self, table_name: String, entry: Entry) -> i256;
        fn update(
            &mut self,
            table_name: String,
            entry: Entry,
            condition: Condition,
        ) -> i256;
        fn remove(&mut self, table_name: String, condition: Condition) -> i256;
        fn desc(&self, table_name: String) -> (String, String);
    }
}

#[liquid::contract]
mod asset_test {
    use super::{table::*, *};

    #[liquid(event)]
    struct RegisterEvent {
        ret_code: i256,
        #[liquid(indexed)]
        account: String,
        #[liquid(indexed)]
        asset_value: u128,
    }

    #[liquid(event)]
    struct TransferEvent {
        ret_code: i256,
        #[liquid(indexed)]
        from: String,
        #[liquid(indexed)]
        to: String,
        value: u128,
    }

    #[liquid(storage)]
    struct AssetTableTest {
        table: storage::Value<Table>,
    }

    #[liquid(methods)]
    impl AssetTableTest {
        pub fn new(&mut self) {
            self.table.initialize(Table::at("0x1001".parse().unwrap()));
            self.table.createTable(
                String::from("t_asset").clone(),
                String::from("account").clone(),
                String::from("asset_value").clone(),
            );
        }

        pub fn select(&mut self, account: String) -> (bool, u128) {
            let cmp_triple = CompareTriple {
                lvalue: String::from("account"),
                rvalue: account,
                cmp: Comparator::EQ(0),
            };
            let mut compare_fields = Vec::new();
            compare_fields.push(cmp_triple);
            let cond = Condition {
                cond_fields: compare_fields,
            };

            let entries = self.table.select(String::from("t_asset"), cond).unwrap();

            if entries.len() < 1 {
                return (false, Default::default());
            }

            return (
                true,
                u128::from_str_radix(&entries[0].fileds[0].value.clone(), 10)
                    .ok()
                    .unwrap(),
            );
        }

        pub fn register(&mut self, account: String, asset_value: u128) -> i256 {
            let ret_code: i256;
            let (ok, _) = self.select(account.clone());
            if ok == false {
                let kv0 = KVField {
                    key: String::from("account"),
                    value: account.clone(),
                };
                let kv1 = KVField {
                    key: String::from("asset_value"),
                    value: asset_value.to_string(),
                };
                let mut kv_fields = Vec::new();
                kv_fields.push(kv0);
                kv_fields.push(kv1);
                let entry = Entry { fileds: kv_fields };
                let result = self.table.insert(String::from("t_asset"), entry).unwrap();

                if result == 1.into() {
                    ret_code = 0.into();
                } else {
                    ret_code = (-2).into();
                }
            } else {
                ret_code = (-1).into();
            }
            let ret = ret_code.clone();
            self.env().emit(RegisterEvent {
                ret_code,
                account,
                asset_value,
            });
            return ret;
        }

        pub fn transfer(&mut self, from: String, to: String, value: u128) -> i256 {
            let mut ret_code: i256 = 0.into();
            let (ok, from_value) = self.select(from.clone());
            if ok == true.into() {
                ret_code = (-1).into();
                self.env().emit(TransferEvent {
                    ret_code,
                    from,
                    to,
                    value,
                });
                return (-1).into();
            }

            let (ret, to_value) = self.select(to.clone());
            if ret != true {
                ret_code = (-2).into();
                self.env().emit(TransferEvent {
                    ret_code,
                    from,
                    to,
                    value,
                });
                return (-2).into();
            }

            if from_value < value.clone() {
                ret_code = (-3).into();
                self.env().emit(TransferEvent {
                    ret_code,
                    from,
                    to,
                    value,
                });
                return (-3).into();
            }

            if to_value.clone() + value.clone() < to_value.clone() {
                ret_code = (-3).into();
                self.env().emit(TransferEvent {
                    ret_code,
                    from,
                    to,
                    value,
                });
                return (-4).into();
            }

            let from_u = self.update(from.clone(), from_value - value.clone());
            if from_u != 1.into() {
                ret_code = (-5).into();
                self.env().emit(TransferEvent {
                    ret_code,
                    from,
                    to,
                    value,
                });
                return (-5).into();
            }

            let r = self.update(to.clone(), to_value.clone() + value.clone());
            self.env().emit(TransferEvent {
                ret_code,
                from,
                to,
                value,
            });
            return r;
        }

        pub fn update(&mut self, account: String, value: u128) -> i256 {
            let kv0 = KVField {
                key: String::from("asset_value"),
                value: value.to_string(),
            };
            let mut kv_fields = Vec::new();
            kv_fields.push(kv0);

            let entry = Entry { fileds: kv_fields };

            let cmp_triple = CompareTriple {
                lvalue: String::from("account"),
                rvalue: account,
                cmp: Comparator::EQ(0),
            };
            let mut compare_fields = Vec::new();
            compare_fields.push(cmp_triple);
            let cond = Condition {
                cond_fields: compare_fields,
            };

            let r = self
                .table
                .update(String::from("t_asset"), entry, cond)
                .unwrap();
            return r;
        }
    }
}
