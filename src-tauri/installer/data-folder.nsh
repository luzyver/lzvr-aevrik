; Included before Tauri's standard pages: data selection is the first Setup page.
; Keep this extension small; Tauri still owns install/upgrade/uninstall behavior.
!include nsDialogs.nsh
!include LogicLib.nsh
!include FileFunc.nsh

Var AevrikDataFolder
Var AevrikDataInput

Page custom AevrikDataPage AevrikDataLeave

Function AevrikDataPage
  ; Upgrades must never offer to replace the existing workspace preference.
  IfFileExists "$APPDATA\id.luzyver.aevrik.desktop\data-location.json" skip_data
  IfFileExists "$APPDATA\id.luzyver.aevrik.desktop\aevrik.sqlite" skip_data
  IfFileExists "$APPDATA\dev.aevrik.desktop\aevrik.sqlite" skip_data
  ${GetOptions} $CMDLINE "/UPDATE" $0
  ${IfNot} ${Errors}
    Goto skip_data
  ${EndIf}
  ${If} $AevrikDataFolder == ""
    ${GetOptions} $CMDLINE "/DATA=" $AevrikDataFolder
    ${If} $AevrikDataFolder == ""
      StrCpy $AevrikDataFolder "$APPDATA\id.luzyver.aevrik.desktop"
    ${EndIf}
  ${EndIf}
  nsDialogs::Create 1018
  Pop $0
  ${If} $0 == error
    Abort
  ${EndIf}
  GetDlgItem $0 $HWNDPARENT 1037
  SendMessage $0 ${WM_SETTEXT} 0 "STR:Choose data folder"
  GetDlgItem $0 $HWNDPARENT 1038
  SendMessage $0 ${WM_SETTEXT} 0 "STR:Choose where Aevrik stores its settings, runtimes and browser data."
  ${NSD_CreateLabel} 0 0 100% 36u "Keep this folder separate from the application folder. Choose an empty folder or an existing Aevrik workspace. Existing files will not be moved."
  Pop $0
  ${NSD_CreateLabel} 0 48u 100% 12u "Data folder:"
  Pop $0
  ${NSD_CreateDirRequest} 0 64u 78% 14u "$AevrikDataFolder"
  Pop $AevrikDataInput
  ${NSD_CreateBrowseButton} 80% 64u 20% 14u "Browse..."
  Pop $0
  ${NSD_OnClick} $0 AevrikDataBrowse
  nsDialogs::Show
  Return
  skip_data:
  Abort
FunctionEnd

Function AevrikDataBrowse
  Pop $0
  ${NSD_GetText} $AevrikDataInput $AevrikDataFolder
  nsDialogs::SelectFolderDialog "Choose Aevrik data folder" "$AevrikDataFolder"
  Pop $0
  ${If} $0 != error
    ${NSD_SetText} $AevrikDataInput $0
  ${EndIf}
FunctionEnd

Function AevrikDataLeave
  ${NSD_GetText} $AevrikDataInput $AevrikDataFolder
  ; Only local absolute paths; the Rust validator also checks ownership, locks,
  ; empty/existing workspace rules, write access and resolved junction paths.
  StrCpy $0 $AevrikDataFolder 2 1
  ${If} $0 != ":\"
    MessageBox MB_ICONSTOP|MB_OK "Choose an absolute folder on a local drive, such as D:\AevrikData."
    Abort
  ${EndIf}
FunctionEnd

!macro NSIS_HOOK_POSTINSTALL
  ; Silent installation also supports /DATA="D:\Data folder". Existing saved
  ; choices always win, even when this parameter is present during an upgrade.
  ${If} $AevrikDataFolder == ""
    ${GetOptions} $CMDLINE "/DATA=" $AevrikDataFolder
    ${If} $AevrikDataFolder == ""
      StrCpy $AevrikDataFolder "$APPDATA\id.luzyver.aevrik.desktop"
    ${EndIf}
  ${EndIf}
  ; A trailing backslash before a quote would escape that quote in Windows argv.
  ; Add a final path component instead of manipulating Unicode/JSON in NSIS.
  ClearErrors
  ${If} ${Silent}
    ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --configure-data-directory "$AevrikDataFolder\." --silent' $0
  ${Else}
    ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --configure-data-directory "$AevrikDataFolder\."' $0
  ${EndIf}
  ${If} ${Errors}
  ${OrIf} $0 != 0
    SetErrorLevel 1
    Abort "Aevrik could not save its data folder. Run Setup again after correcting the folder or closing Aevrik."
  ${EndIf}
!macroend
