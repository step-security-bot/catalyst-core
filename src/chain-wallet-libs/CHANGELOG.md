# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

# Added

## Cordova-android | Java | C
- vote_proposal_new_public
- vote_proposal_new_private

# Deprecated

- proposal_new: In favour of the specific functions for each case. This
function takes an enum, which currently only can be used to cast public
votes (the internal function still uses rust enums, this is only for non-rust
apis).

### Added

#### wallet-js

- Add Ed25519Extended generation from seed
- Key signing and verification.
- Add the other kinds of private keys: 
  - Ed25519

## [0.5.0-pre4] - 2020-10-13

### Added

#### wallet-js

- Key pair generation support.
- Symmetric encryption and decryption support.

## [0.5.0-pre3] - 2020-09-04

### Fixed

- Decryption function now returns an error if the authentication fails.

### Added

#### wallet-cordova

- iOS support for the import key and decryption functions.

## [0.5.0-pre2] - 2020-08-18

### Fixed
- Wrong secret key type was used when recovering from mnemonics.

## [0.5.0-pre1] - 2020-08-18
### Added

- New utxo store.
- Allow recovering from single free utxo keys.
- Custom symmetric encryption/decryption module.

## [0.4.0] - 2020-07-08

## [0.4.0-pre3] - 2020-06-22

## [0.4.0-pre2] - 2020-06-04

## [0.4.0-pre1] - 2020-06-03

## [0.3.1] - 2020-22-05

## [0.3.0] - 2020-05-01

## [0.2.0] - 2020-04-15

## [0.1.0] - 2020-04-10