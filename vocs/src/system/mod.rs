/// A system that will reach an equilibrium after enough iterations.
/// These types of systems can be executed in parallel, refining the result.
/// For example, the Minecraft lighting algorithm is a good example.
pub mod incremental;