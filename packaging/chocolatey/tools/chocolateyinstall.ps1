$ErrorActionPreference = 'Stop'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$url64 = 'https://github.com/Brunopaun/easy-docker/releases/download/v0.1.6/easy-docker-x86_64-pc-windows-msvc.zip'
$checksum64 = 'dd9fb792d44d996a91ce4e3b86b0ab5e0c7694940b1e2c1d37855ce4b3caa245'

$packageArgs = @{
  packageName    = 'easy-docker'
  unzipLocation  = $toolsDir
  url64bit       = $url64
  checksum64     = $checksum64
  checksumType64 = 'sha256'
}

Install-ChocolateyZipPackage @packageArgs
