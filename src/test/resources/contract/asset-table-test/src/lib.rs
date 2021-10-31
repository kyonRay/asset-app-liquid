#![cfg_attr(not(feature = "std"), no_std)]
#![feature(unboxed_closures, fn_traits)]

use liquid::storage;
use liquid_lang as liquid;

#[liquid::interface(name = auto)]
mod entry {
    extern "liquid" {
        fn getInt(&self, key: String) -> i256;
        fn getUint(&self, key: String) -> u256;
        fn getAddress(&self, key: String) -> Address;
        fn getString(&self, key: String) -> String;

        fn setI256(&mut self, key: String, value: i256);
        fn setU256(&mut self, key: String, value: u256);
        fn setAddress(&mut self, key: String, value: Address);
        fn setString(&mut self, key: String, value: String);
    }
}

#[liquid::interface(name = auto)]
mod condition {
    extern "liquid" {
        fn EQ(&mut self, value1: String, value2: String);
        fn NE(&mut self, value1: String, value2: String);

        fn GT(&mut self, value1: String, value2: i256);
        fn GE(&mut self, value1: String, value2: i256);
        fn LT(&mut self, value1: String, value2: i256);
        fn LE(&mut self, value1: String, value2: i256);
        fn limit(&mut self, lower: i256, upper: i256);
    }
}

#[liquid::interface(name = auto)]
mod entries {
    use super::entry::*;

    extern "liquid" {
        fn get(&self, value: i256) -> Entry;
        fn size(&self) -> i256;
    }
}

#[liquid::interface(name = auto)]
mod table {
    use super::{condition::*, entries::*, entry::*};

    extern "liquid" {
        fn select(&self, condition: Condition) -> Entries;
        fn insert(&mut self, entry: Entry) -> i256;
        fn update(&mut self, entry: Entry, condition: Condition) -> i256;
        fn remove(&mut self, condition: Condition) -> i256;

        fn newEntry(&self) -> Entry;
        fn newCondition(&self) -> Condition;
    }
}

#[liquid::interface(name = auto)]
mod table_factory {
    use super::table::*;

    extern "liquid" {
        fn openTable(&self, name: String) -> Table;
        fn createTable(
            &mut self,
            name: String,
            primary_key: String,
            fields: String,
        ) -> i256;
    }
}

#[liquid::contract]
mod asset_test {
    use super::{table_factory::*, *};

    #[liquid(event)]
    struct RegisterEvent {
        ret_code: i256,
        #[liquid(indexed)]
        account: String,
        #[liquid(indexed)]
        asset_value: u256,
    }

    #[liquid(event)]
    struct TransferEvent {
        ret_code: i256,
        #[liquid(indexed)]
        from: String,
        #[liquid(indexed)]
        to: String,
        value: u256,
    }

    #[liquid(storage)]
    struct AssetTableTest {
        table_factory: storage::Value<TableFactory>,
    }

    #[liquid(methods)]
    impl AssetTableTest {
        pub fn new(&mut self) {
            self.table_factory
                .initialize(TableFactory::at("0x1001".parse().unwrap()));
            self.table_factory.createTable(
                String::from("t_asset").clone(),
                String::from("account").clone(),
                String::from("asset_value").clone(),
            );
        }

        pub fn select(&mut self, account: String) -> (i256, u256) {
            let table = self
                .table_factory
                .openTable(String::from("t_asset").clone())
                .unwrap();
            let mut cond = table.newCondition().unwrap();
            cond.EQ(String::from("account"), account);

            let entries = table.select(cond).unwrap();
            if entries.size().unwrap() < 1.into() {
                return ((-1).into(), 0.into());
            }
            let entry = entries.get(0.into()).unwrap();
            return (
                0.into(),
                entry.getUint(String::from("asset_value").clone()).unwrap(),
            );
        }

        pub fn register(&mut self, account: String, asset_value: u256) -> i256 {
            let ret_code: i256;
            let (ok, _) = self.select(account.clone());
            if ok != 0.into() {
                let mut table = self
                    .table_factory
                    .openTable(String::from("t_asset").clone())
                    .unwrap();
                let mut entry = table.newEntry().unwrap();
                entry.setString(String::from("account").clone(), account.clone());
                entry.setU256(String::from("asset_value").clone(), asset_value.clone());
                let count = table.insert(entry).unwrap();
                if count == 1.into() {
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

        pub fn transfer(&mut self, from: String, to: String, value: u256) -> i256 {
            let mut ret_code: i256 = 0.into();
            let (ret, from_value) = self.select(from.clone());
            if ret != 0.into() {
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
            if ret != 0.into() {
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

        pub fn update(&mut self, account: String, value: u256) -> i256 {
            let mut table = self
                .table_factory
                .openTable(String::from("t_asset").clone())
                .unwrap();
            let mut entry = table.newEntry().unwrap();
            entry.setU256(String::from("asset_value").clone(), value);
            let mut cond = table.newCondition().unwrap();
            cond.EQ(String::from("account"), account);
            let r = table.update(entry, cond).unwrap();
            return r;
        }
    }
}
