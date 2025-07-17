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

    pub fn get_kv_direct<T>(&self, _key: &str) -> Result<Option<T>, DatabaseError> {
        Ok(None)
    }

    pub fn get_kv<T>(&self, _key: &str) -> Result<Option<T>, DatabaseError> {
        Ok(None)
    }

    pub fn set_kv_direct<T>(&self, _key: &str, _value: T) -> Result<(), DatabaseError> {
        Ok(())
    }

    pub fn set_kv<T>(&self, _key: &str) -> Result<(), DatabaseError> {
        Ok(())
    }
}
