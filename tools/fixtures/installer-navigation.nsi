; Harmless NSIS regression fixture. No application, registry, shortcuts or profile writes.
Unicode true
!include MUI2.nsh
!include nsDialogs.nsh
Name "Aevrik navigation test"
OutFile "${OUTPUT}"
RequestExecutionLevel user
InstallDir "$EXEDIR\Default application"
Page custom DataPage
!insertmacro MUI_PAGE_WELCOME
; Tauri checks previous installations here before skipping its reinstall page.
Page custom SkippedPage
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_LANGUAGE "English"

Function DataPage
  nsDialogs::Create 1018
  Pop $0
  ${NSD_CreateDirRequest} 0 30u 100% 14u "$EXEDIR\Data folder"
  Pop $0
  nsDialogs::Show
FunctionEnd

Function SkippedPage
  Sleep ${TRANSITION_DELAY}
  Abort
FunctionEnd

Section
  ; A navigation test must stop before this section can execute.
  FileOpen $0 "$EXEDIR\unexpected-install.txt" w
  FileWrite $0 "Navigation clicked past the directory page"
  FileClose $0
SectionEnd
