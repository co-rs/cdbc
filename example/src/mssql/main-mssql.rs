use mco::std::time::Time;

fn main() -> cdbc::Result<()> {
    #[derive(Debug,serde::Serialize)]
    pub struct BizActivity {
        pub id: Option<String>,
        pub name: Option<String>,
        pub delete_flag: Option<i32>,
        pub create_time: Option<Time>
    }

    Ok(())
}


#[cfg(test)]
mod test {

    #[test]
    fn test_stream_mysql() -> cdbc::Result<()> {

        Ok(())
    }
}