$ErrorActionPreference = 'Stop'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$url64 = 'https://github.com/Brunopaun/easy-docker/releases/download/v0.1.7/easy-docker-x86_64-pc-windows-msvc.zip'
$checksum64 = '0b086ab731e699c76b587213edaa9a27047d946d4ec4f88f5a76995a6e98c383'

$packageArgs = @{
  packageName    = 'easy-docker'
  unzipLocation  = $toolsDir
  url64bit       = $url64
  checksum64     = $checksum64
  checksumType64 = 'sha256'
}

Install-ChocolateyZipPackage @packageArgs
