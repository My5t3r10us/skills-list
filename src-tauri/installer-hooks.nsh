!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Adding skills-list CLI to the user PATH"
  nsExec::ExecToLog `powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "$$installDir = [System.IO.Path]::GetFullPath('$INSTDIR').TrimEnd('\'); $$path = [Environment]::GetEnvironmentVariable('Path', 'User'); $$parts = @($$path -split ';' | ForEach-Object { $$_.Trim() } | Where-Object { $$_ }); if (-not ($$parts | Where-Object { $$_.TrimEnd('\') -ieq $$installDir })) { $$parts += $$installDir; [Environment]::SetEnvironmentVariable('Path', ($$parts -join ';'), 'User') }"`
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Removing skills-list CLI from the user PATH"
  nsExec::ExecToLog `powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "$$installDir = [System.IO.Path]::GetFullPath('$INSTDIR').TrimEnd('\'); $$path = [Environment]::GetEnvironmentVariable('Path', 'User'); $$parts = @($$path -split ';' | ForEach-Object { $$_.Trim() } | Where-Object { $$_ -and ($$_.TrimEnd('\') -ine $$installDir) }); [Environment]::SetEnvironmentVariable('Path', ($$parts -join ';'), 'User')"`
!macroend
