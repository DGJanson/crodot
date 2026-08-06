// This handler parses a NONE coding response
// (coding response has its own handler in the fileutils mod)
// Most parsing for now is looking for code blocks.

use crate::{AppState, tui::llmresponse::LlmResponseScreenState};

pub fn handle_llm_response(state: &AppState, screen_state: &mut LlmResponseScreenState) {
    let blocks = scan_code_blocks(&state.llm_response);
    screen_state.code_blocks = blocks;
    screen_state.selected_block = 0;
    screen_state.selected = false;
}

fn scan_code_blocks(input: &str) -> Vec<(usize, usize)> {
    let mut blocks: Vec<(usize, usize)> = Vec::new();
    let code_block_mark = "```";
    let mut idx = 0;
    // Scan input for the code blocks
    // Note that all finds are relative
    while let Some(start) = input[idx..].find(code_block_mark) {
        let start = start + idx + code_block_mark.len();
        // We need to check if the code block has a language specifier.
        // So the block starts from the next \n
        let next_break = input[start..].find("\n").unwrap_or(0);
        let start = start + next_break + 1;
        // Find end of block
        if let Some(end_idx) = input[start..].find(code_block_mark) {
            // Found block, add to resuls
            let end = start + end_idx;
            blocks.push((start, end));
            idx = end + code_block_mark.len()
        } else {
            break;
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::scan_code_blocks;

    #[test]
    fn test_scan() {
        let input = r#"
        This is a string
        ```qualifier
        first code block
        ```

        some whitespace

        ```
        Another code block
        With some
        whitespace



        
        ```   

        Some final
        remarks
        
        "#;

        let blocks = scan_code_blocks(input);
        assert_eq!(blocks.len(), 2);

        let input_2 = "```
12
```

```rust
34
```";
        let blocks_2 = scan_code_blocks(input_2);
        assert_eq!(blocks_2.len(), 2);
        assert_eq!(blocks_2.first().unwrap().0, 4);
        assert_eq!(blocks_2.first().unwrap().1, 7);
        let first_index = blocks_2.get(1).unwrap().0;
        let second_index = blocks_2.get(1).unwrap().1;
        assert_eq!(&input_2[first_index..second_index], "34\n");
    }
}
