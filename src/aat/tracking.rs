use crate::Face;
use crate::buffer::Buffer;
use crate::plan::ShapePlan;

pub fn apply(plan: &ShapePlan, face: &Face, buffer: &mut Buffer) -> Option<()> {
    let trak_mask = plan.trak_mask;

    let ptem = face.points_per_em?;
    if ptem <= 0.0 {
        return None;
    }

    let trak = face.trak?;

    if !buffer.have_positions {
        buffer.clear_positions();
    }

    if buffer.direction.is_horizontal() {
        let tracking = trak.hor_tracking(ptem)?;
        let advance_to_add = tracking;
        let offset_to_add = tracking / 2;
        foreach_grapheme!(buffer, start, end, {
            if buffer.info[start].mask & trak_mask != 0 {
                buffer.pos[start].x_advance += advance_to_add;
                buffer.pos[start].x_offset += offset_to_add;
            }
        });
    } else {
        let tracking = trak.ver_tracking(ptem)?;
        let advance_to_add = tracking;
        let offset_to_add = tracking / 2;
        foreach_grapheme!(buffer, start, end, {
            if buffer.info[start].mask & trak_mask != 0 {
                buffer.pos[start].y_advance += advance_to_add;
                buffer.pos[start].y_offset += offset_to_add;
            }
        });
    }

    Some(())
}
