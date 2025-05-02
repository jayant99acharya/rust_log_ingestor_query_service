pub mod db;
pub mod error;
pub mod handlers;
pub mod models;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod api_tests {
    include!("tests/api_tests.rs");
}

#[cfg(test)]
mod performance_tests {
    include!("tests/performance_tests.rs");
}
