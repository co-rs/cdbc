use std::iter::{Extend, IntoIterator};

#[derive(Debug, Default)]
pub struct MssqlQueryResult {
    pub(super) rows_affected: u64,
}

impl MssqlQueryResult {
    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }

    /// this un support
    #[deprecated]
    pub fn last_insert_id(&self) -> i64{
        //not allow
        -1
    }
}

impl Extend<MssqlQueryResult> for MssqlQueryResult {
    fn extend<T: IntoIterator<Item = MssqlQueryResult>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
        }
    }
}

