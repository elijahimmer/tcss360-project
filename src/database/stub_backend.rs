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
        table: &str,
        _key: &str,
    ) -> Result<Option<T>, DatabaseError> {
        Ok(None)
    }

    pub fn get_kv_table<T>(&self, table: &str, _key: &str) -> Result<Option<T>, DatabaseError> {
        Ok(None)
    }

    pub fn get_kv_direct<T>(&self, _key: &str) -> Result<Option<T>, DatabaseError> {
        Ok(None)
    }

    pub fn get_kv<T>(&self, _key: &str) -> Result<Option<T>, DatabaseError> {
        Ok(None)
    }

    pub fn get_kv_table_direct_or_default<T>(&self, _table: &str, _key: &str, default: T) -> T {
        default
    }

    pub fn get_kv_table_or_default<T>(
        &self,
        _table: &str,
        _key: &str,
        default: T,
    ) -> Result<Option<T>, DatabaseError> {
        default
    }

    pub fn set_kv_direct<T>(&self, _key: &str, _value: T) -> Result<(), DatabaseError> {
        Ok(())
    }

    pub fn set_kv<T>(&self, _key: &str) -> Result<(), DatabaseError> {
        Ok(())
    }

    pub fn set_kv_table_direct<T>(&self, _key: &str, _value: T) -> Result<(), DatabaseError> {
        Ok(())
    }

    pub fn set_kv_table<T>(&self, _key: &str) -> Result<(), DatabaseError> {
        Ok(())
    }
}
