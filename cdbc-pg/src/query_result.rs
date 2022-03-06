use std::iter::{Extend, IntoIterator};

#[derive(Debug, Default)]
pub struct PgQueryResult {
    pub(super) rows_affected: u64,
}

impl PgQueryResult {
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

impl Extend<PgQueryResult> for PgQueryResult {
    fn extend<T: IntoIterator<Item = PgQueryResult>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
        }
    }
}
