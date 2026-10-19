$ErrorActionPreference = 'Stop'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$binary = Join-Path $toolsDir "easy-docker.exe"

if (Test-Path $binary) {
  Remove-Item $binary -Force
}
