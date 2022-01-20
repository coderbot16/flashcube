/// Implements the lighting "kernel" function that propagates light within and between cells.

// For each 2x2x2 cell, the layout is:
// first 16 bits: z layer 0
// second 16 bits: z layer 1
// each layer (x, y): (0, 0), (1, 0), (0, 1), (1, 1)
//
//     e----+----f
//    / 6  / 7  /|  ^
//   /----+----+7|  |
//  / 2  / 3  /|/|  Y
// a----+----b | |
// | 2  |  3 |/|5.  ^
// +----+----|1|/  /
// | 0  |  1 | /  Z
// c----+----d
//      X ->
// 
// X left to right
// Y up/down
// Z depth

fn kernel(mut light: u4x16, mut opacity: u4x16, emission: u4x16, neighbor_light: SplitDirectional<u4x16>) -> u4x16 {
	opacity = u4x16::saturating_add(u4x16::splat(1));

	// Fast path: No need to pull in light from neighbors here, if the max light value is
	// already emitted at all 16 locations.
	if (emission.0 == u64::MAX) {
		return u4x16(u64::MAX);
	}

	// shouldn't run more than 4 times
	// TODO: Cleaner loop code?
	loop {
		// 6x max (24 ops each)
		// 2x xmid (5 ops each)

		let incoming_x = u4x16::max (
			xmid(light, neighbor_light.plus_x),
			xmid(neighbor_light.minus_x, light)
		);

		let incoming_y = u4x16::max (
			ymid(light, neighbor_light.plus_y),
			ymid(neighbor_light.minus_y, light)
		);

		let incoming_z = u4x16::max (
			zmid(light, neighbor_light.plus_z),
			zmid(neighbor_light.minus_z, light)
		);

		let incoming = incoming_z.max(incoming_x.max(incoming_y));

		// incoming - opacity
		let new = u4x16::max(
			u4x16::saturating_sub(incoming, opacity),
			emission
		);

		if (new == light) {
			return light;
		}

		light = new;
	}
}

// Need: saturating_sub, saturating_add


// Input:
//         x0                x1
//     +----+----+      +----+----+
//    /    /  M /|     / N  /  . /|
//   /----+----+M|    /----+----+ |
//  /    /  M /|/|   / N  /  . /|/|
// +----+----+M| |  +----+----+ | |
// | .  |  M |/|M|  | N  |  . |/| |
// +----+----|M|/   +----+----| |/
// | .  |  M | /    | N  |  . | /
// +----+----+`     +----+----+`
//
// Output:
//        xmid
//     +----+----+
//    / M  /  N /|
//   /----+----+N|
//  / M  /  N /|/|
// +----+----+N|N|
// | M  |  N |/| |
// +----+----|N|/
// | M  |  N | /
// +----+----+`
// 
// Takes (LM,NR) and returns (MN)
//
// Ops: 5
// - 2x and
// - 1x lsl
// - 1x lsr
// - 1x or
fn xmid(x0: u4x16, x1: u4x16) -> u4x16 {
	// All of the M elements in x0 are the odd elements,
	// and all of the N elements in x1 are the even elements.
	const ODD_MASK:  u64 = 0xf0f0_f0f0_f0f0_f0f0;
	const EVEN_MASK: u64 = 0x0f0f_0f0f_0f0f_0f0f;

	// converts odd to even
	let m = (x0.0 & ODD_MASK) >> 4;

	// converts even to odd
	let n = (x1.0 & EVEN_MASK) << 4;

	// merge odd and even parts
	return u4x16(m | n);
}

// Input:
//                          z1
//                     +----+----+
//                    /    /    /|
//                   /----+----+ |
//        z0        / N  /  N /|/|
//     +----+----+ +----+----+N| |
//    / M  /  M /| | N  |  N |/| |
//   /----+----+M| +----+----|N|/
//  /    /    /|/| | N  |  N | /
// +----+----+ | | +----+----+`
// |    |    |/|M|
// +----+----| |/
// |    |    | /
// +----+----+`
//
// Output:
//         zmid
//     +----+----+
//    / N  /  N /|
//   /----+----+N|
//  / M  /  M /|/|
// +----+----+ |N|
// | M  |  M |/| |
// +----+----| |/
// | M  |  M | /
// +----+----+`
fn zmid(z0: u4x16, z1: u4x16) -> u4x16 {
	const HI_MASK: u64 = 0xFFFF_0000_FFFF_0000;
	const LO_MASK: u64 = 0x0000_FFFF_0000_FFFF;

	//     z0: 0xMMMM_0000_MMMM_0000
	//     z1: 0x0000_NNNN_0000_NNNN
	// result: 0xNNNN_MMMM_NNNN_MMMM

	let m = (z0.0 & HI_MASK) >> 16;
	let n = (z1.0 & LO_MASK) << 16;

	return u4x16(m | n);
}


// Input:
//          y1
//     +----+----+
//    /    /    /|
//   /----+----+ |
//  /    /    /|/|
// +----+----+ | |
// |    |    |/|N|
// +----+----|N|/
// | N  |  N | /
// +----+----+`
//
//        y0
//     +----+----+
//    / M  /  M /|
//   /----+----+M|
//  / M  /  M /|/|
// +----+----+M| |
// | M  |  M |/| |
// +----+----| |/
// |    |    | /
// +----+----+`
//
// Output:
//        ymid
//     +----+----+
//    / N  / N  /|
//   /----+----+ |
//  / N  / N  /|/|
// +----+----+ | |
// | N  | N  |/| |
// +----+----| |/
// | M  | M  | /
// +----+----+`


fn ymid(y0: u4x16, y1: u4x16) -> u4x16 {
	//     y0: 0xMM00_MM00_MM00_MM00
	//     y1: 0x00NN_00NN_00NN_00NN
	// result: 0xNNMM_NNMM_NNMM_NNMM

	const HI_MASK: u64 = 0xFF00_FF00_FF00_FF00;
	const LO_MASK: u64 = 0x00FF_00FF_00FF_00FF;

	let m = (y0.0 & HI_MASK) >> 8;
	let n = (y1.0 & LO_MASK) << 8;

	return u4x16(m | n);
}


// Cube template for docs:
//     +----+----+
//    /    /    /|
//   /----+----+ |
//  /    /    /|/|
// +----+----+ | |
// |    |    |/| |
// +----+----| |/
// |    |    | /
// +----+----+`
