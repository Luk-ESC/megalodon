use crate::grid::EMPTY;

#[expect(clippy::comparison_chain)]
pub fn smart_resize(
    buffer: &mut Vec<u32>,
    (old_width, old_height): (usize, usize),
    (new_width, new_height): (usize, usize),
) {
    let old_length = old_width * old_height;
    let new_length = new_width * new_height;

    buffer.reserve_exact(new_length);
    assert_eq!(buffer.len(), old_width * old_height);

    if new_width < old_width {
        // Remove pixels from the right side
        let mut next_insert = 0;

        for row in 0..old_height {
            let row_offset = row * old_width;
            buffer.copy_within(row_offset..row_offset + new_width, next_insert);
            next_insert += new_width;
        }

        buffer.truncate(old_height * new_width);
    } else if new_width > old_width {
        // Add empty space to the right side
        let new_space = (new_width - old_width) * old_height;
        buffer.resize(old_length + new_space, EMPTY);

        for row in (0..old_height).rev() {
            let new_start = row * new_width;
            let row_offset = row * old_width;
            buffer.copy_within(row_offset..row_offset + old_width, new_start);
            buffer[new_start + old_width..new_start + new_width].fill(EMPTY);
        }
    }

    assert_eq!(buffer.len(), old_height * new_width);

    if new_height < old_height {
        let height_diff = old_height - new_height;
        buffer.drain(..new_width * height_diff);
    } else if new_height > old_height {
        let height_diff = new_height - old_height;
        let new_space = new_width * height_diff;

        buffer.resize(new_length, 0x00FF0000);
        buffer.copy_within(..new_width * old_height, new_space);
        buffer[..new_space].fill(EMPTY);
    }

    assert_eq!(buffer.len(), new_length);
}
