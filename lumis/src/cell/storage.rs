// 8x8x8 array of 2x2x2 cells storing nibble data for a 16x16x16 area.
// Cells are stored as array elements for data locality.
struct CellNibbleCube([u32; 512]);