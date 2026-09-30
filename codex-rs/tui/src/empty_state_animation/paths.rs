//! Prisma AIRS and Palo Alto Networks mark geometry for the logo morph.
//!
//! Paths use absolute M/L/H/V/C/Z commands only; the field builder parses nothing else.
//! The renderer rests on the second field and morphs into the first mid-spin.

// Three 45° bars, traced from the owner-supplied 600×600 raster and simplified to its corners.
// The bars touch at (300, 192) and (300, 407), so one outline covers the whole mark.
pub(super) const PALO_ALTO: (&str, [f64; 4]) = (
    "M228 122L299 192L370 122L441 191L264 370L300 406L477 229L549 299L371 477L300 407L227 477L157 407L335 228L300 192L122 370L50 300Z",
    [50.0, 122.0, 549.0, 477.0],
);
// Same coordinates as ../airs_onboarding/prisma-airs-mark.svg (191×256 viewBox). The two
// filled regions leave the triangular center open to the terminal's own background.
pub(super) const AIRS: (&str, [f64; 4]) = (
    "M128 0L0 128L128 256L128 171L68 171L128 52ZM128 52L191 171L128 171Z",
    [0.0, 0.0, 191.0, 256.0],
);
