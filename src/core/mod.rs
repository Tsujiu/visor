//! Módulo central de regras de negócio, navegação e geometria.

pub mod geometry;
pub mod image_cache;
pub mod image_ops;
pub mod navigation;

#[cfg(test)]
pub mod stress_tests;

pub use image_cache::ImageCache;
