pub mod statement_cache;
pub mod ustr;
pub mod async_stream;

#[macro_export]
macro_rules! try_stream {
    ($($block:tt)*) => {
        crate::utils::async_stream::TryAsyncStream::new(move |mut sender| async move {
            macro_rules! r#yield {
                ($v:expr) => {{
                   // let _ = futures_util::sink::SinkExt::send(&mut sender, Ok($v)).await;
                }}
            }

            $($block)*
        })
    }
}
