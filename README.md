# `bgt60trxx`
An async and no_std rust library to interface via SPI with the XENSIV™ BGT60TRxx 60 GHz FMCW radar sensors from Infineon.

## Supported Sensors
- BGT60TR13C
- BGT60UTR11AIP

## What works
Nothing yet.

## Basic Usage
TODO.

## Modules

### BGT60TR13C
There are a couple of ready-made modules with the BGT60TR13C that include all required supporting components:
- CY8CKIT-062S2-AI with [CY8CKIT-062S2-AI-PASSTHROUGH](https://github.com/thedevleon/CY8CKIT-062S2-AI-PASSTHROUGH) firmware
- SHIELDBGT60TR13CTOBO1
- DEMOBGT60TR13CTOBO1 (which includes SHIELDBGT60TR13CTOBO1)
- KITCSKBGT60TR13CTOBO1 (from which you only need the radar wing)

### BGT60UTR11AIP
- todo