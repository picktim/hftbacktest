use std::{
    fs::File,
    io::{Error, Write},
    path::Path,
};

use crate::{
    depth::MarketDepth,
    types::{Interface, Recorder},
};

/// Provides recording of the backtesting strategy's state values, which are needed to compute
/// performance metrics.
pub struct BacktestRecorder {
    values: Vec<Vec<(i64, f32, f64, f64, f64, i32, f64, f64)>>,
}

impl Recorder for BacktestRecorder {
    type Error = Error;

    fn record<Q, MD, I>(&mut self, hbt: &mut I) -> Result<(), Self::Error>
    where
        Q: Sized + Clone,
        I: Interface<Q, MD>,
        MD: MarketDepth,
    {
        let timestamp = hbt.current_timestamp();
        for asset_no in 0..hbt.num_assets() {
            let depth = hbt.depth(asset_no);
            let mid = (depth.best_bid() + depth.best_ask()) / 2.0;
            let state_values = hbt.state_values(asset_no);
            let values = unsafe { self.values.get_unchecked_mut(asset_no) };
            values.push((
                timestamp,
                mid,
                state_values.balance,
                state_values.position,
                state_values.fee,
                state_values.trade_num,
                state_values.trade_amount,
                state_values.trade_qty,
            ));
        }
        Ok(())
    }
}

impl BacktestRecorder {
    /// Constructs an instance of `BacktestRecorder`.
    pub fn new<Q, MD, I>(hbt: &I) -> Self
    where
        Q: Sized + Clone,
        I: Interface<Q, MD>,
        MD: MarketDepth,
    {
        Self {
            values: {
                let mut vec = Vec::with_capacity(hbt.num_assets());
                for _ in 0..hbt.num_assets() {
                    vec.push(Vec::new());
                }
                vec
            },
        }
    }

    /// Saves record data into a CSV file at the specified path. It creates a separate CSV file for
    /// each asset, with the filename `record_{asset_no}.csv`.
    /// The columns are `timestamp`, `mid`, `balance`, `position`, `fee`, `trade_num`,
    /// `trade_amount`, `trade_qty`.
    pub fn to_csv<P>(&self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        for (asset_no, values) in self.values.iter().enumerate() {
            let file_path = path.as_ref().join(format!("record_{asset_no}.csv"));
            let mut file = File::create(file_path)?;
            for (timestamp, mid, balance, position, fee, trade_num, trade_amount, trade_qty) in
                values
            {
                write!(
                    file,
                    "{},{},{},{},{},{},{},{}\n",
                    timestamp, mid, balance, position, fee, trade_num, trade_amount, trade_qty
                )?;
            }
        }
        Ok(())
    }
}
