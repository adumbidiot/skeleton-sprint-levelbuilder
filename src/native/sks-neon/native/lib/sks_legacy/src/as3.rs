use sks::block::Block;
use crate::LEVEL_WIDTH;

pub fn encode_as3(level: &str, data: &[Block]) -> String {
    data.iter()
        .enumerate()
        .fold(String::new(), |mut out, (i, block)| {
            if i % LEVEL_WIDTH == 0 {
                out += &format!("lvlArray[{}][{}] = [", level, i / LEVEL_WIDTH);
            }

            match block {
                Block::Note { .. } => {
                    out += "\"";
                    out += &block.as_lbl();
                    out += "\"";
                }
                _ => {
                    out += &block.as_lbl();
                }
            }

            if i % LEVEL_WIDTH == LEVEL_WIDTH - 1 {
                out += "];\n"
            } else {
                out += ", ";
            }

            out
        })
}
