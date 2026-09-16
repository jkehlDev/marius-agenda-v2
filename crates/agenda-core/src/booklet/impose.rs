//! Saddle-stitch booklet imposition helpers.
//! Reading-order pages are 0-indexed; output sheets are A4 landscape (2×A5).

/// Pad page count up to the next multiple of 4 (one sheet = 4 page sides).
pub fn pad_count_to_signature(count: usize) -> usize {
    if count == 0 {
        return 4;
    }
    let rem = count % 4;
    if rem == 0 { count } else { count + (4 - rem) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImposedSheetSide {
    pub left: Option<usize>,
    pub right: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImposedSheet {
    pub index: usize,
    pub front: ImposedSheetSide,
    pub back: ImposedSheetSide,
}

/// Classic booklet imposition for N pages (N multiple of 4, 1-based math then converted).
pub fn build_imposition(page_count: usize) -> Vec<ImposedSheet> {
    let n = pad_count_to_signature(page_count);
    let sheets = n / 4;
    let mut result = Vec::with_capacity(sheets);

    for s in 0..sheets {
        let front_left_1 = n - 2 * s;
        let front_right_1 = 2 * s + 1;
        let back_left_1 = 2 * s + 2;
        let back_right_1 = n - 2 * s - 1;

        let to_idx = |one_based: usize| -> Option<usize> {
            let zero = one_based - 1;
            if zero < page_count { Some(zero) } else { None }
        };

        result.push(ImposedSheet {
            index: s,
            front: ImposedSheetSide {
                left: to_idx(front_left_1),
                right: to_idx(front_right_1),
            },
            back: ImposedSheetSide {
                left: to_idx(back_left_1),
                right: to_idx(back_right_1),
            },
        });
    }

    result
}

/// Booklet side from reading-order 0-based index: odd pages (1,3,…) = recto.
pub fn booklet_side_class(reading_index: usize) -> &'static str {
    if (reading_index + 1) % 2 == 1 {
        "is-recto"
    } else {
        "is-verso"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imposition_eight_pages() {
        let sheets = build_imposition(8);
        assert_eq!(sheets.len(), 2);
        assert_eq!(sheets[0].front.left, Some(7));
        assert_eq!(sheets[0].front.right, Some(0));
    }
}
