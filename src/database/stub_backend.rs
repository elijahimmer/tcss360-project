use bevy::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {}

#[derive(Resource)]
pub struct Database;

impl Database {
    pub fn open() -> Result<Self, DatabaseError> {
        Ok(Self)
    }

    pub fn get_kv_table_direct<T>(
        &self,
        _table: &str,
        _key: &str,
    ) -> Result<Option<T>, DatabaseError> {
        Ok(None)
    }

    pub fn get_kv_table<T>(&self, _table: &str, _key: &str) -> Result<Option<T>, DatabaseError> {
        Ok(None)
    }

    pub fn get_kv_direct<T>(&self, _key: &str) -> Result<Option<T>, DatabaseError> {
        Ok(None)
    }

    pub fn get_kv<T>(&self, _key: &str) -> Result<Option<T>, GetKvError> {
        Ok(None)
    }

    pub fn get_kv_table_direct_or_default<T, U: Into<T>>(
        &self,
        _table: &str,
        _key: &str,
        default: U,
    ) -> T {
        default.into()
    }

    pub fn get_kv_table_or_default<T, U: Into<T>>(
        &self,
        _table: &str,
        _key: &str,
        default: U,
    ) -> T {
        default.into()
    }

    pub fn set_kv_direct<T>(&self, _key: &str, _value: T) -> Result<(), DatabaseError> {
        Ok(())
    }

    pub fn set_kv<T>(&self, _key: &str) -> Result<(), SetKvError> {
        Ok(())
    }

    pub fn set_kv_table_direct<T>(
        &self,
        _table: &str,
        _key: &str,
        _value: T,
    ) -> Result<(), DatabaseError> {
        Ok(())
    }

    pub fn set_kv_table<T>(
        &self,
        _table: &str,
        _key: &str,
        _value: T,
    ) -> Result<(), DatabaseError> {
        Ok(())
    }
}

pub type GetKvError = DatabaseError;
pub type SetKvError = DatabaseError;
