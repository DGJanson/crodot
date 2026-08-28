# LLM Response Screen

Displays the response returned by the LLM, with options to handle or inspect code blocks.

Layout:
- Left panel: info and commands  
- Right panel: response text (or coding summary)

Modes:
- **Main**: View response  
- **Handle Prompt**: Options to start new or continue prompt  
- **Handle File**: Choose a file from coding suggestions  
- **Function**: Show navigation menu  

Shortcuts (Main):
- **Space**: Open the function menu  
- **p**: Open prompt handling menu (new/continue)  
- **f**: Open file selection for code responses  
- **j / k**: Scroll down/up in the response  
- **K**: Jump to top of response  
- **[ / ]**: Navigate between code blocks (when not in coding mode)  
- **y**: Yank selected code block to clipboard  
- **r**: Toggle raw output display  

Shortcuts (Handle Prompt):
- **n**: Start a new prompt (reset chat)  
- **c**: Continue the previous chat  

Shortcuts (Function Menu):
- **l**: Go to LLM Interaction screen  
- **f**: Open File Manager  
- **m**: Return to Main Menu  

Shortcuts (Handle File):
- **0-9**: Enter file index to open  
- **Backspace**: Reset index input  
- **Enter**: Confirm selection