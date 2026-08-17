# Menu Screen

The menu screen is the main entry point of the Crodot TUI. It shows project information and provides keyboard-driven navigation and configuration.

## Main View

Displays:
- **Project Name**
- **Project Directory**
- **Project File Location**

## Shortcuts

- **Space**: Open function selection menu  
- **p**: Open project settings menu  

## Function Selection

In the function selection popup:
- **l**: Switch to LLM prompt screen  
- **r**: Switch to LLM response screen  
- **f**: Switch to file manager screen  
- **q**: Quit Crodot  

Press **Esc** to close the popup and return to the main view.

## Project Settings

In the project settings popup:
- **n**: Edit project name  
- **l**: Edit project file save location  
- **r**: Edit project root directory  
- **s**: Save project to file  
- **d**: Manage prompt directories  

Press **Esc** to close the project settings popup.

## Managing Prompt Directories

In the prompt directories view:
- Lists current prompt directories (index, path, description).  
- **a**: Add a new prompt directory (enter path, then description).  
- **r**: Remove a prompt directory by index.  

Press **Esc** to cancel and return to project settings.

## Editing Mode

While editing a field:
- **Type** to enter text  
- **Enter** to confirm  
- **Esc** to cancel  
- **Backspace** to delete characters  