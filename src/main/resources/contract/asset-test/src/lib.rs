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

#[liquid::interface(name = auto)]
mod kv_table {
    use super::*;

    extern "liquid" {
        fn createTable(
            &mut self,
            table_name: String,
            key: String,
            value_fields: String,
        ) -> i256;
        fn get(&self, table_name: String, key: String) -> (bool, Entry);
        fn set(&mut self, table_name: String, key: String, entry: Entry) -> i256;
    }
}

#[liquid::contract]
mod asset_test {
    use super::{kv_table::*, *};

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
        table: storage::Value<KvTable>,
    }

    #[liquid(methods)]
    impl AssetTableTest {
        pub fn new(&mut self) {
            self.table
                .initialize(KvTable::at("/sys/kv_storage".parse().unwrap()));
            self.table.createTable(
                String::from("t_asset").clone(),
                String::from("account").clone(),
                String::from("asset_value").clone(),
            );
        }

        pub fn select(&mut self, account: String) -> (bool, u128) {
            if let Some((result, entry)) =
            (*self.table).get(String::from("t_asset"), account)
            {
                return (
                    result,
                    u128::from_str_radix(&entry.fileds[0].value.clone(), 10)
                        .ok()
                        .unwrap(),
                );
            }
            return (false, Default::default());
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
                let result = (*self.table)
                    .set(String::from("t_asset"), account.clone(), entry)
                    .unwrap();

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

            let r = (*self.table)
                .set(String::from("t_asset"), account, entry)
                .unwrap();
            return r;
        }
    }
}
