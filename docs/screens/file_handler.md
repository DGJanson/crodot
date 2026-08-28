# Handle Code Screen

Review and write coding responses to files on disk.

Layout:
- Left panel: metadata (filename, mode, description)  
- Right panel: file contents preview  

Modes:
- **Main**: View content and metadata  
- **Handle**: Show write or back options  
- **Success**: Display write-confirmation popup  
- **FileLoc**: Override output file path input  

Shortcuts (Main):
- **j / k**: Scroll down/up in the preview  
- **K**: Jump to top of preview  
- **h**: Open handle menu (write or back)  
- **L**: Return to LLM Response screen  
- **f**: Enter file path override mode  

Shortcuts (Handle Menu):
- **w**: Write the file to disk  
- **b**: Back to LLM Response screen  

Shortcuts (FileLoc Override):
- **Type**: Append to override path  
- **Backspace**: Remove last character  
- **Delete**: Cancel override and return to Main  
- **Enter**: Confirm override path  

Success Popup:
- Any key: Acknowledge and return to LLM Response screen